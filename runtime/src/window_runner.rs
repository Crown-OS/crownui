use std::time::{Duration, Instant};

use geometry::Size;
use graphics::{CreateRenderer, GraphicsBackend, TextSystem};
use layouting::LayoutEngine;
use view::{LayoutState, TextCache, Ui, View};
use windowing::{
    CursorIcon, Decorations, Platform, ResizeEdge, WindowControl, WindowEvent, WindowId,
};

use crate::{
    config::AppConfig,
    frame::build_batch,
    render_thread::{RenderMessage, RenderThread},
    resize_band,
    runner::RunError,
};

const LONG_PRESS_POLL: Duration = Duration::from_millis(16);
const DEFAULT_REFRESH: Duration = Duration::from_micros(16_667);

/// One window of a running app: its UI, layout and text state, and the render thread presenting it.
pub(crate) struct WindowRunner<G: GraphicsBackend, L: LayoutEngine> {
    config: AppConfig,
    ui: Ui,
    layout: LayoutState<L>,
    text: TextCache<G::TextSystem>,
    window: Option<WindowId>,
    render: Option<RenderThread<G::TextLayout>>,
    bridges: crate::bridges::Bridges,
    frames: crate::frame_stats::FrameHistory,
    last_ui_us: u32,
    text_input: Option<(geometry::Rect, bool)>,
    idle_inhibited: bool,
    layer_height: Option<windowing::LayerExtent>,
    refresh: Duration,
    next_animation_frame: Instant,
    /// When the next display frame will be sampled, from the last frame the render thread
    /// presented; animation-frame callbacks run for that time.
    frame_due: Option<Duration>,
    last_animation_frame: Duration,
    render_wakes_ui: bool,
    ui_cursor: CursorIcon,
    resize_hover: Option<ResizeEdge>,
}

impl<G: GraphicsBackend, L: LayoutEngine> WindowRunner<G, L> {
    pub(crate) fn new(root: impl View, engine: L, config: AppConfig) -> Result<Self, RunError> {
        let mut text = TextCache::new(G::create_text_system()?);
        for (bytes, alias) in &config.fonts {
            text.system_mut()
                .register_font(bytes.clone(), alias.as_deref())?;
        }
        let mut ui = Ui::new(root);
        ui.set_color_scheme(config.color_scheme);
        Ok(Self {
            config,
            ui,
            layout: LayoutState::new(engine),
            text,
            window: None,
            render: None,
            bridges: crate::bridges::Bridges::default(),
            frames: crate::frame_stats::FrameHistory::default(),
            last_ui_us: 0,
            text_input: None,
            idle_inhibited: false,
            layer_height: None,
            refresh: DEFAULT_REFRESH,
            next_animation_frame: Instant::now(),
            frame_due: None,
            last_animation_frame: Duration::ZERO,
            render_wakes_ui: false,
            ui_cursor: CursorIcon::Default,
            resize_hover: None,
        })
    }

    pub(crate) fn window(&self) -> Option<WindowId> {
        self.window
    }

    /// Creates the platform window and its render thread, then shows the first frame.
    pub(crate) fn open<P>(&mut self, platform: &mut P, epoch: Instant) -> Result<(), RunError>
    where
        P: Platform,
        G: CreateRenderer<P::Surface>,
    {
        let window = platform
            .create_window(self.config.window_attributes())
            .map_err(|error| RunError::Platform(error.to_string()))?;
        self.window = Some(window);
        self.sync_capabilities(platform);
        let surface = platform
            .surface(window)
            .ok_or_else(|| RunError::Platform("CUI0406: the window has no surface".into()))?;
        self.render = Some(RenderThread::spawn::<G, P::Surface>(
            surface,
            self.config.clear_color,
            self.config.corner_shape,
            epoch,
            platform.waker(),
        )?);
        self.bridges
            .start(&self.config, &mut self.ui, &|| platform.waker());
        if self.config.mcp {
            let app_id = self
                .config
                .app_id
                .clone()
                .unwrap_or_else(|| self.config.title.clone());
            let gated = platform.request_agent_access(
                &app_id,
                "Let AI agents read and operate this app's UI over MCP",
            );
            if !gated {
                self.bridges.start_mcp(&self.config.title, platform.waker());
            }
        }
        self.resize(platform);
        self.push_frame(platform, epoch)
    }

    /// Hands the platform's capabilities to the UI: at open, and again whenever the platform
    /// reports an event that may have changed them, such as the compositor withdrawing its blur.
    pub(crate) fn sync_capabilities<P: Platform>(&mut self, platform: &P) {
        let capabilities = platform.capabilities();
        let layer = capabilities.layer_shell && self.config.layer.is_some();
        self.ui.set_emulate_materials(!capabilities.materials);
        self.ui
            .set_composited_materials(capabilities.composited_materials && layer);
        self.ui.set_platform_capabilities(capabilities);
    }

    pub(crate) fn push_frame<P: Platform>(
        &mut self,
        platform: &mut P,
        epoch: Instant,
    ) -> Result<(), RunError> {
        if !self.ui.needs_frame() && self.render.is_some() {
            self.bridges.after_frame(&mut self.ui);
            self.sync_platform_services(platform);
            return Ok(());
        }
        let started = Instant::now();
        let (batch, cursor) = build_batch(
            &mut self.ui,
            &mut self.layout,
            &mut self.text,
            epoch.elapsed(),
        );
        self.last_ui_us = crate::frame_stats::micros(started.elapsed());
        self.receive_reports();
        if let Some(cursor) = cursor {
            self.ui_cursor = cursor;
        }
        if let (Some(cursor), None, Some(window)) = (
            cursor,
            self.resize_hover,
            self.window.and_then(|id| platform.window_mut(id)),
        ) {
            window.set_cursor(cursor);
        }
        self.bridges.after_frame(&mut self.ui);
        self.sync_platform_services(platform);
        if batch.is_empty() {
            return Ok(());
        }
        let delivered = self
            .render
            .as_ref()
            .is_some_and(|render| render.send(RenderMessage::Batch(Box::new(batch))));
        if delivered {
            Ok(())
        } else {
            Err(RunError::Platform(
                "CUI0405: the render thread stopped".into(),
            ))
        }
    }

    /// Takes in the frames the render thread presented since the last call.
    fn receive_reports(&mut self) {
        let Some(render) = &self.render else {
            return;
        };
        let mut received = false;
        for report in render.reports() {
            if let Some(refresh) = report.refresh.filter(|refresh| !refresh.is_zero()) {
                self.refresh = refresh;
            }
            self.frame_due = Some(report.sampled_at + self.refresh);
            self.frames.push(inspect::FrameStats {
                frame: report.frame,
                update_us: 0,
                layout_us: self.last_ui_us,
                record_us: report.record_us,
                render_us: report.render_us,
            });
            received = true;
        }
        if received {
            self.bridges.set_frames(&self.frames);
        }
    }

    fn sync_platform_services<P: Platform>(&mut self, platform: &mut P) {
        for request in self.ui.take_clipboard_requests() {
            match request {
                view::ClipboardRequest::Copy(text) => platform.set_clipboard_text(text),
                view::ClipboardRequest::Paste => platform.request_clipboard_text(),
                view::ClipboardRequest::Offer(items) => platform.set_clipboard(items),
                view::ClipboardRequest::Read(request, mime_types) => {
                    platform.read_clipboard(request, mime_types)
                }
            }
        }
        if let (Some(source), Some(window)) = (self.ui.take_drag_request(), self.window) {
            platform.start_drag(window, source);
        }
        if let Some(window) = self.window {
            for request in self.ui.take_app_icon_requests() {
                match request {
                    view::AppIconRequest::Set(app_id, placement) => {
                        platform.set_app_icon(window, &app_id, placement)
                    }
                    view::AppIconRequest::Remove(app_id) => {
                        platform.remove_app_icon(window, &app_id)
                    }
                    view::AppIconRequest::Activate(app_id) => {
                        platform.activate_app_icon(window, &app_id)
                    }
                }
            }
        }
        if let Some(window) = self.window.and_then(|id| platform.window_mut(id)) {
            for request in self.ui.take_window_requests() {
                match request {
                    view::WindowRequest::Move => window.begin_move(),
                    view::WindowRequest::Minimize => window.minimize(),
                    view::WindowRequest::ToggleMaximize => window.toggle_maximize(),
                }
            }
        }
        let inhibit = self.ui.idle_inhibit_request();
        if inhibit != self.idle_inhibited
            && let Some(window) = self.window.and_then(|id| platform.window_mut(id))
        {
            window.set_idle_inhibited(inhibit);
            self.idle_inhibited = inhibit;
        }
        let layer_height = self.ui.layer_height_request();
        if layer_height != self.layer_height
            && let (Some(height), Some(window)) = (
                layer_height,
                self.window.and_then(|id| platform.window_mut(id)),
            )
        {
            window.set_layer_height(height);
            self.layer_height = layer_height;
        }
        let wanted = self.ui.text_input_request();
        if wanted != self.text_input
            && let Some(window) = self.window.and_then(|id| platform.window_mut(id))
        {
            window.set_text_input(
                wanted.map(|(cursor_area, secure)| windowing::TextInputState {
                    cursor_area,
                    surrounding_text: None,
                    content_purpose: if secure {
                        windowing::TextInputPurpose::Password
                    } else {
                        windowing::TextInputPurpose::Normal
                    },
                }),
            );
            self.text_input = wanted;
        }
    }

    /// Steps `request_animation_frame` callbacks once per display frame. While the render thread
    /// presents frames, each one it presents wakes this thread to run them for the time the next
    /// frame is sampled at, so their changes land one per frame on the render thread's clock.
    /// Otherwise a timer at the refresh interval starts them, and the frame they produce brings
    /// the render thread's pacing back.
    fn run_animation_frames(&mut self, epoch: Instant) {
        let wanted = self.ui.wants_animation_frame();
        self.wake_ui_after_frames(wanted);
        let elapsed = epoch.elapsed();
        let frame_due = self
            .frame_due
            .take()
            .filter(|due| *due + self.refresh > elapsed);
        let now = Instant::now();
        let time = match frame_due {
            _ if !wanted => return,
            Some(due) => due,
            None if now >= self.next_animation_frame => elapsed,
            None => return,
        };
        let time = time.max(self.last_animation_frame);
        self.last_animation_frame = time;
        self.ui.run_animation_frames(time);
        let fallback = if frame_due.is_some() {
            self.refresh + self.refresh / 2
        } else {
            self.refresh
        };
        self.next_animation_frame = now + fallback;
    }

    fn wake_ui_after_frames(&mut self, wanted: bool) {
        if wanted == self.render_wakes_ui {
            return;
        }
        if let Some(render) = &self.render
            && render.send(RenderMessage::AnimationFrames(wanted))
        {
            self.render_wakes_ui = wanted;
        }
    }

    fn resize<P: Platform>(&mut self, platform: &mut P) {
        let Some(window) = self.window.and_then(|id| platform.window(id)) else {
            return;
        };
        let logical = window.logical_size();
        let physical = window.inner_size();
        let scale = window.scale_factor().get() as f32;
        self.ui
            .set_viewport(Size::new(logical.width as f32, logical.height as f32));
        self.ui.set_scale_factor(scale);
        self.ui.set_output_size(
            window
                .output_size()
                .map(|size| Size::new(size.width as f32, size.height as f32)),
        );
        self.bridges.set_scale(f64::from(scale));
        if let Some(render) = self.render.as_ref().filter(|_| window.is_configured()) {
            render.send(RenderMessage::Viewport {
                width: physical.width,
                height: physical.height,
                scale,
            });
        }
    }

    /// Routes an event to this window's UI.
    pub(crate) fn window_event<P: Platform>(&mut self, platform: &mut P, event: &WindowEvent) {
        match event {
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged(_) => self.resize(platform),
            WindowEvent::Focused(focused) => self.bridges.set_window_focused(*focused),
            WindowEvent::Visibility { visibility, .. } => {
                if let Some(render) = &self.render {
                    render.send(RenderMessage::Paused(visibility.is_obscured()));
                }
            }
            _ => {}
        }
        if self.grab_resize_band(platform, event) {
            return;
        }
        self.ui.handle_event(event);
        if let Some(accepted) = self.ui.take_drag_acceptance() {
            platform.accept_drag(accepted.as_ref());
        }
    }

    /// Shows the resize cursor along a client-decorated window's edges and starts a resize on
    /// a primary press there; returns whether it took the event from the UI.
    fn grab_resize_band<P: Platform>(&mut self, platform: &mut P, event: &WindowEvent) -> bool {
        if self.config.decorations != Decorations::Client {
            return false;
        }
        let Some(window) = self.window.and_then(|id| platform.window_mut(id)) else {
            return false;
        };
        let edge = match event {
            WindowEvent::PointerMove(pointer) | WindowEvent::PointerDown(pointer) => window
                .is_floating()
                .then(|| resize_band::edge_at(pointer.position, window.logical_size()))
                .flatten(),
            WindowEvent::PointerLeave(_) => None,
            _ => return false,
        };
        if edge != self.resize_hover {
            self.resize_hover = edge;
            window.set_cursor(edge.map_or(self.ui_cursor, ResizeEdge::cursor));
        }
        match (event, edge) {
            (WindowEvent::PointerDown(pointer), Some(edge))
                if pointer.button == Some(input::PointerButton::Primary) =>
            {
                window.begin_resize(edge);
                true
            }
            _ => false,
        }
    }

    pub(crate) fn agent_access(
        &mut self,
        decision: &windowing::AgentAccess,
        waker: windowing::Waker,
    ) {
        if !self.config.mcp {
            return;
        }
        match decision {
            windowing::AgentAccess::Granted(_) => self.bridges.start_mcp(&self.config.title, waker),
            windowing::AgentAccess::Denied => tracing::info!("agent access denied by the platform"),
            windowing::AgentAccess::Revoked => self.bridges.stop_mcp(),
        }
    }

    /// Whether the window should close now (see [`Ui::take_close_request`]).
    pub(crate) fn take_close_request(&mut self) -> bool {
        self.ui.take_close_request()
    }

    /// Delivers finished tasks and agent requests.
    pub(crate) fn poll(&mut self) {
        self.bridges.poll(&mut self.ui);
    }

    /// Polls, steps animation frames and pushes a frame; returns when the loop must wake next.
    pub(crate) fn before_wait<P: Platform>(
        &mut self,
        platform: &mut P,
        epoch: Instant,
    ) -> Result<Option<Instant>, RunError> {
        self.poll();
        self.receive_reports();
        self.run_animation_frames(epoch);
        self.push_frame(platform, epoch)?;
        let long_press = self
            .ui
            .has_active_pointers()
            .then(|| Instant::now() + LONG_PRESS_POLL);
        let animation = self
            .ui
            .wants_animation_frame()
            .then_some(self.next_animation_frame);
        Ok(long_press.into_iter().chain(animation).min())
    }
}
