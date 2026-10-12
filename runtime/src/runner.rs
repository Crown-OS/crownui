use std::time::{Duration, Instant};

use geometry::Size;
use graphics::{CreateRenderer, GraphicsBackend, GraphicsError, TextSystem};
use layouting::LayoutEngine;
use view::{LayoutState, TextCache, Ui, View};
use windowing::{ApplicationHandler, ControlFlow, Platform, WindowControl, WindowEvent, WindowId};

use crate::{
    config::AppConfig,
    frame::build_batch,
    render_thread::{RenderMessage, RenderThread},
};

const LONG_PRESS_POLL: Duration = Duration::from_millis(16);
const DEFAULT_REFRESH: Duration = Duration::from_micros(16_667);

/// Why an app could not start.
#[derive(Debug, thiserror::Error)]
pub enum RunError {
    /// The graphics backend failed to initialize.
    #[error(transparent)]
    Graphics(#[from] GraphicsError),
    /// The platform failed.
    #[error("platform error: {0}")]
    Platform(String),
    /// A thread could not be spawned.
    #[error("could not spawn a thread: {0}")]
    Thread(#[from] std::io::Error),
}

/// Drives a CrownUI app on any [`Platform`] with any graphics backend and layout engine:
/// the UI thread runs here, rendering happens on a dedicated render thread.
pub struct Runner<G: GraphicsBackend, L: LayoutEngine> {
    config: AppConfig,
    ui: Ui,
    layout: LayoutState<L>,
    text: TextCache<G::TextSystem>,
    window: Option<WindowId>,
    render: Option<RenderThread<G::TextLayout>>,
    epoch: Instant,
    bridges: crate::bridges::Bridges,
    failure: Option<RunError>,
    frames: crate::frame_stats::FrameHistory,
    last_ui_us: u32,
    text_input: Option<(geometry::Rect, bool)>,
    idle_inhibited: bool,
    refresh: Duration,
    next_animation_frame: Instant,
}

impl<G: GraphicsBackend, L: LayoutEngine> Runner<G, L> {
    /// Prepares `root` to run with `config`; nothing is shown until the platform resumes.
    pub fn new(root: impl View, engine: L, config: AppConfig) -> Result<Self, RunError> {
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
            epoch: Instant::now(),
            bridges: crate::bridges::Bridges::default(),
            failure: None,
            frames: crate::frame_stats::FrameHistory::default(),
            last_ui_us: 0,
            text_input: None,
            idle_inhibited: false,
            refresh: DEFAULT_REFRESH,
            next_animation_frame: Instant::now(),
        })
    }

    /// The error that stopped the app, if any.
    pub fn take_failure(&mut self) -> Option<RunError> {
        self.failure.take()
    }

    fn fail<P: Platform>(&mut self, platform: &mut P, error: RunError) {
        tracing::error!("{error}");
        self.failure = Some(error);
        platform.exit();
    }

    fn push_frame<P: Platform>(&mut self, platform: &mut P) {
        if !self.ui.needs_frame() && self.render.is_some() {
            self.bridges.after_frame(&mut self.ui);
            self.sync_platform_services(platform);
            return;
        }
        let started = Instant::now();
        let (batch, cursor) = build_batch(
            &mut self.ui,
            &mut self.layout,
            &mut self.text,
            self.epoch.elapsed(),
        );
        self.last_ui_us = crate::frame_stats::micros(started.elapsed());
        if let Some(render) = &self.render {
            for report in render.reports() {
                if let Some(refresh) = report.refresh.filter(|refresh| !refresh.is_zero()) {
                    self.refresh = refresh;
                }
                self.frames.push(inspect::FrameStats {
                    frame: report.frame,
                    update_us: 0,
                    layout_us: self.last_ui_us,
                    record_us: report.record_us,
                    render_us: report.render_us,
                });
            }
        }
        self.bridges.set_frames(self.frames.snapshot());
        if let (Some(cursor), Some(window)) =
            (cursor, self.window.and_then(|id| platform.window_mut(id)))
        {
            window.set_cursor(cursor);
        }
        self.bridges.after_frame(&mut self.ui);
        self.sync_platform_services(platform);
        if batch.is_empty() {
            return;
        }
        let delivered = self
            .render
            .as_ref()
            .is_some_and(|render| render.send(RenderMessage::Batch(Box::new(batch))));
        if !delivered {
            self.fail(
                platform,
                RunError::Platform("CUI0405: the render thread stopped".into()),
            );
        }
    }

    fn sync_platform_services<P: Platform>(&mut self, platform: &mut P) {
        for request in self.ui.take_clipboard_requests() {
            match request {
                view::ClipboardRequest::Copy(text) => platform.set_clipboard_text(text),
                view::ClipboardRequest::Paste => platform.request_clipboard_text(),
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
        let inhibit = self.ui.idle_inhibit_request();
        if inhibit != self.idle_inhibited
            && let Some(window) = self.window.and_then(|id| platform.window_mut(id))
        {
            window.set_idle_inhibited(inhibit);
            self.idle_inhibited = inhibit;
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

    /// Steps `request_animation_frame` callbacks once per display refresh; the frame they produce
    /// is then paced by the render thread's own frame callbacks.
    fn run_animation_frames(&mut self) {
        let now = Instant::now();
        if !self.ui.wants_animation_frame() || now < self.next_animation_frame {
            return;
        }
        self.ui.run_animation_frames(self.epoch.elapsed());
        let cadence = self.next_animation_frame + self.refresh;
        self.next_animation_frame = if cadence > now {
            cadence
        } else {
            now + self.refresh
        };
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
        self.bridges.set_scale(f64::from(scale));
        if let Some(render) = &self.render {
            render.send(RenderMessage::Viewport {
                width: physical.width,
                height: physical.height,
                scale,
            });
        }
    }
}

impl<P, G, L> ApplicationHandler<P> for Runner<G, L>
where
    P: Platform,
    G: GraphicsBackend + CreateRenderer<P::Surface>,
    L: LayoutEngine,
{
    fn resumed(&mut self, platform: &mut P) {
        let window = match platform.create_window(self.config.window_attributes()) {
            Ok(window) => window,
            Err(error) => return self.fail(platform, RunError::Platform(error.to_string())),
        };
        self.window = Some(window);
        self.ui
            .set_emulate_materials(!platform.capabilities().materials);
        self.ui.set_platform_capabilities(platform.capabilities());
        let Some(surface) = platform.surface(window) else {
            return self.fail(
                platform,
                RunError::Platform("CUI0406: the window has no surface".into()),
            );
        };
        match RenderThread::spawn::<G, P::Surface>(
            surface,
            self.config.clear_color,
            self.config.corner_shape,
            self.epoch,
        ) {
            Ok(render) => self.render = Some(render),
            Err(error) => return self.fail(platform, RunError::Thread(error)),
        }
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
        self.push_frame(platform);
    }

    fn window_event(&mut self, platform: &mut P, _window: WindowId, event: WindowEvent) {
        match &event {
            WindowEvent::CloseRequested => {
                self.render = None;
                platform.exit();
                return;
            }
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged(_) => self.resize(platform),
            WindowEvent::Focused(focused) => self.bridges.set_window_focused(*focused),
            WindowEvent::Visibility { visibility, .. } => {
                if let Some(render) = &self.render {
                    render.send(RenderMessage::Paused(visibility.is_obscured()));
                }
            }
            _ => {}
        }
        self.ui.handle_event(&event);
        if let Some(accepted) = self.ui.take_drag_acceptance() {
            platform.accept_drag(accepted.as_deref());
        }
    }

    fn agent_access(&mut self, platform: &mut P, decision: windowing::AgentAccess) {
        match decision {
            windowing::AgentAccess::Granted(_) => {
                self.bridges.start_mcp(&self.config.title, platform.waker())
            }
            windowing::AgentAccess::Denied => tracing::info!("agent access denied by the platform"),
            windowing::AgentAccess::Revoked => self.bridges.stop_mcp(),
        }
    }

    fn proxy_wake_up(&mut self, platform: &mut P) {
        self.bridges.poll(&mut self.ui);
        self.push_frame(platform);
    }

    fn about_to_wait(&mut self, platform: &mut P) {
        self.bridges.poll(&mut self.ui);
        self.run_animation_frames();
        self.push_frame(platform);
        let long_press = self
            .ui
            .has_active_pointers()
            .then(|| Instant::now() + LONG_PRESS_POLL);
        let animation = self
            .ui
            .wants_animation_frame()
            .then_some(self.next_animation_frame);
        let control_flow = match long_press.into_iter().chain(animation).min() {
            Some(deadline) => ControlFlow::WaitUntil(deadline),
            None => ControlFlow::Wait,
        };
        platform.set_control_flow(control_flow);
    }
}
