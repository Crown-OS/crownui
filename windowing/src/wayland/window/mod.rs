mod handles;
mod presentation;
mod render_surface;
pub(crate) mod scaling;
mod shell;
pub(crate) mod state;

pub use handles::SurfaceHandle;
pub use render_surface::RenderSurface;
use sctk::reexports::protocols::xdg::shell::client::xdg_toplevel::ResizeEdge as XdgResizeEdge;
use sctk::{
    compositor::FrameCallbackData,
    reexports::client::{
        Connection, Proxy, QueueHandle,
        protocol::{wl_compositor::WlCompositor, wl_seat::WlSeat, wl_surface::WlSurface},
    },
    reexports::protocols::wp::{
        idle_inhibit::zv1::client::{
            zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1,
            zwp_idle_inhibitor_v1::ZwpIdleInhibitorV1,
        },
        presentation_time::client::wp_presentation::WpPresentation,
    },
    shell::xdg::window::WindowDecorations,
};

use crate::{
    CursorIcon, Decorations, LayerExtent, LayerOptions, Material, ResizeEdge, TextInputState,
    WindowAttributes, WindowControl, WindowId,
    dpi::{LogicalSize, PhysicalSize, ScaleFactor},
    wayland::{
        color_management::SurfaceColorTag,
        error::Error,
        event_loop::WaylandPlatform,
        output::Output,
        protocols::background_effects::{BackgroundEffect, BackgroundEffectSurface},
        util::no_events::NoEvents,
    },
};
use scaling::SurfaceScaling;
pub(crate) use shell::ShellSurface;

pub(crate) fn surface_window_id(surface: &WlSurface) -> WindowId {
    WindowId::from_raw(surface.id().as_ptr() as u64)
}

fn xdg_resize_edge(edge: ResizeEdge) -> XdgResizeEdge {
    match edge {
        ResizeEdge::Top => XdgResizeEdge::Top,
        ResizeEdge::Bottom => XdgResizeEdge::Bottom,
        ResizeEdge::Left => XdgResizeEdge::Left,
        ResizeEdge::Right => XdgResizeEdge::Right,
        ResizeEdge::TopLeft => XdgResizeEdge::TopLeft,
        ResizeEdge::TopRight => XdgResizeEdge::TopRight,
        ResizeEdge::BottomLeft => XdgResizeEdge::BottomLeft,
        ResizeEdge::BottomRight => XdgResizeEdge::BottomRight,
    }
}

fn xdg_decorations(decorations: Decorations) -> WindowDecorations {
    match decorations {
        Decorations::Server => WindowDecorations::RequestServer,
        Decorations::Client => WindowDecorations::ClientOnly,
    }
}

/// An xdg toplevel or a layer surface owned by the event loop; reach it through
/// [`WaylandPlatform::window`] and [`WaylandPlatform::window_mut`].
pub struct Window {
    shell: ShellSurface,
    layer: Option<LayerOptions>,
    /// The primary output's logical size when last seen, which `LayerExtent::Output` resolves to.
    output_size: Option<(u32, u32)>,
    idle_inhibitor: Option<ZwpIdleInhibitorV1>,
    idle_inhibit_manager: Option<ZwpIdleInhibitManagerV1>,
    compositor: WlCompositor,
    presentation: Option<WpPresentation>,
    srgb_requested: bool,
    color: Option<SurfaceColorTag>,
    text_input: Option<TextInputState>,
    text_input_changed: bool,
    background_effect: Option<BackgroundEffectSurface>,
    scaling: SurfaceScaling,
    /// Whether a render surface took over committing the buffer's geometry.
    frames_carry_geometry: bool,
    connection: Connection,
    queue_handle: QueueHandle<WaylandPlatform>,
    floating_size: LogicalSize,
    floating: bool,
    maximized: bool,
    logical_size: LogicalSize,
    scale_factor: ScaleFactor,
    cursor: CursorIcon,
    last_button_press: Option<(WlSeat, u32)>,
    configured: bool,
    redraw_requested: bool,
    frame_callback_pending: bool,
    cursor_changed: bool,
    id: WindowId,
}

impl WaylandPlatform {
    pub fn create_window(&mut self, attributes: WindowAttributes) -> WindowId {
        let qh = &self.queue_handle;
        let surface = self.compositor_state.create_surface(qh);
        let id = surface_window_id(&surface);
        let scaling = SurfaceScaling::new(self.scaling.as_ref(), &surface, id, qh);
        let background_effect = self
            .crown
            .material
            .as_ref()
            .map(|material| material.effect_surface(&surface, qh));
        if let (Some(material), Some(effect_surface)) = (&attributes.material, &background_effect) {
            effect_surface.apply(&BackgroundEffect::from_material(material, None));
        }

        let mut logical_size = attributes.inner_size;
        let output_size = shell::primary_output_size(self.outputs());
        let layer = attributes
            .layer
            .filter(|_| self.standard.layer_shell.is_some());
        let shell = match (&layer, &self.standard.layer_shell) {
            (Some(options), Some(layer_shell)) => {
                let fallback = (attributes.inner_size.width, attributes.inner_size.height);
                let requested = options.requested_size(output_size, fallback);
                logical_size = LogicalSize::new(
                    if requested.0 == 0 {
                        fallback.0
                    } else {
                        requested.0
                    },
                    if requested.1 == 0 {
                        fallback.1
                    } else {
                        requested.1
                    },
                );
                ShellSurface::Layer(shell::create_layer(
                    layer_shell,
                    surface,
                    options,
                    requested,
                    qh,
                ))
            }
            _ => {
                let toplevel = self.xdg_shell.create_window(
                    surface,
                    xdg_decorations(attributes.decorations),
                    qh,
                );
                toplevel.set_title(attributes.title);
                if let Some(app_id) = attributes.app_id {
                    toplevel.set_app_id(app_id);
                }
                toplevel.set_min_size(attributes.min_size.map(|size| (size.width, size.height)));
                toplevel.set_max_size(attributes.max_size.map(|size| (size.width, size.height)));
                if attributes.maximized {
                    toplevel.set_maximized();
                }
                ShellSurface::Toplevel(toplevel)
            }
        };
        shell.commit();

        self.windows.insert(
            id,
            Window {
                shell,
                layer,
                output_size,
                idle_inhibitor: None,
                idle_inhibit_manager: self.standard.idle_inhibit.clone(),
                compositor: self.compositor_state.wl_compositor().clone(),
                presentation: self.standard.presentation.clone(),
                srgb_requested: false,
                color: None,
                text_input: None,
                text_input_changed: false,
                background_effect,
                scaling,
                frames_carry_geometry: false,
                connection: self.connection.clone(),
                queue_handle: qh.clone(),
                floating_size: logical_size,
                floating: !attributes.maximized,
                maximized: attributes.maximized,
                logical_size,
                scale_factor: ScaleFactor::IDENTITY,
                cursor: CursorIcon::Default,
                last_button_press: None,
                configured: false,
                redraw_requested: false,
                frame_callback_pending: false,
                cursor_changed: false,
                id,
            },
        );
        let observer = self
            .window(id)
            .and_then(|window| self.observe_visibility(window).ok());
        if let Some(observer) = observer {
            self.visibility_observers.insert(id, observer);
        }
        self.replay_app_windows(id);
        id
    }

    pub fn window(&self, id: WindowId) -> Option<&Window> {
        self.windows.get(&id)
    }

    pub fn window_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        self.windows.get_mut(&id)
    }

    /// Destroys the window once every [`SurfaceHandle`] to it has been dropped too.
    pub fn close_window(&mut self, id: WindowId) {
        self.visibility_observers.remove(&id);
        if let Some(icons) = self.crown.app_icons.as_mut() {
            icons.forget_window(id);
        }
        self.windows.remove(&id);
    }
}

impl Window {
    pub fn id(&self) -> WindowId {
        self.id
    }

    /// Queues a [`WindowEvent::RedrawRequested`](crate::wayland::WindowEvent::RedrawRequested),
    /// delivered once the compositor is ready for a new frame.
    pub fn request_redraw(&mut self) {
        self.redraw_requested = true;
    }

    /// Call right before presenting from the event-loop thread, so the next redraw is
    /// paced by the compositor's frame callback. Render threads use [`RenderSurface`].
    pub fn pre_present_notify(&mut self) {
        if std::mem::replace(&mut self.frame_callback_pending, true) {
            return;
        }
        let surface = self.shell.wl_surface();
        surface.frame(&self.queue_handle, FrameCallbackData(surface.clone()));
    }

    /// A handle for presenting from any thread; it keeps the surface alive.
    pub fn surface_handle(&self) -> SurfaceHandle {
        SurfaceHandle::new(
            self.shell.clone(),
            self.connection.clone(),
            self.compositor.clone(),
            self.background_effect
                .as_ref()
                .map(BackgroundEffectSurface::proxies),
            self.presentation.clone(),
        )
    }

    /// Frame pacing for a render thread, on an event queue of its own. The render thread then
    /// commits every change of buffer geometry together with the buffer drawn for it.
    pub fn render_surface(&mut self) -> RenderSurface {
        self.frames_carry_geometry = true;
        RenderSurface::new(self.surface_handle(), self.scaling.mapping())
    }

    pub fn inner_size(&self) -> PhysicalSize {
        self.logical_size.to_physical(self.scale_factor)
    }

    pub fn logical_size(&self) -> LogicalSize {
        self.logical_size
    }

    pub fn scale_factor(&self) -> ScaleFactor {
        self.scale_factor
    }

    pub fn wl_surface(&self) -> &WlSurface {
        self.shell.wl_surface()
    }

    /// The layer-surface options, when this window is a layer surface rather than a toplevel.
    pub fn layer(&self) -> Option<&LayerOptions> {
        self.layer.as_ref()
    }

    pub fn set_title(&self, title: impl Into<String>) {
        if let Some(toplevel) = self.shell.toplevel() {
            toplevel.set_title(title);
        }
    }

    pub fn set_app_id(&self, app_id: impl Into<String>) {
        if let Some(toplevel) = self.shell.toplevel() {
            toplevel.set_app_id(app_id);
        }
    }

    pub fn set_min_size(&mut self, size: Option<LogicalSize>) {
        if let Some(toplevel) = self.shell.toplevel() {
            toplevel.set_min_size(size.map(|size| (size.width, size.height)));
            self.request_redraw();
        }
    }

    pub fn set_max_size(&mut self, size: Option<LogicalSize>) {
        if let Some(toplevel) = self.shell.toplevel() {
            toplevel.set_max_size(size.map(|size| (size.width, size.height)));
            self.request_redraw();
        }
    }

    pub fn maximize(&self) {
        if let Some(toplevel) = self.shell.toplevel() {
            toplevel.set_maximized();
        }
    }

    pub fn unmaximize(&self) {
        if let Some(toplevel) = self.shell.toplevel() {
            toplevel.unset_maximized();
        }
    }

    pub fn minimize(&self) {
        if let Some(toplevel) = self.shell.toplevel() {
            toplevel.set_minimized();
        }
    }

    /// Fullscreens on `output`, or on an output of the compositor's choosing.
    pub fn set_fullscreen(&self, output: Option<&Output>) {
        if let Some(toplevel) = self.shell.toplevel() {
            toplevel.set_fullscreen(output.map(|output| &output.proxy));
        }
    }

    pub fn exit_fullscreen(&self) {
        if let Some(toplevel) = self.shell.toplevel() {
            toplevel.unset_fullscreen();
        }
    }

    pub fn cursor(&self) -> CursorIcon {
        self.cursor
    }

    pub fn set_cursor(&mut self, icon: CursorIcon) {
        self.cursor = icon;
        self.cursor_changed = true;
    }

    /// Starts an interactive move; call it while handling a pointer button press.
    pub fn drag_window(&self) {
        if let (Some((seat, serial)), Some(toplevel)) =
            (&self.last_button_press, self.shell.toplevel())
        {
            toplevel.move_(seat, *serial);
        }
    }

    /// Starts an interactive resize; call it while handling a pointer button press.
    pub fn drag_resize_window(&self, edge: ResizeEdge) {
        if let (Some((seat, serial)), Some(toplevel)) =
            (&self.last_button_press, self.shell.toplevel())
        {
            toplevel.resize(seat, *serial, xdg_resize_edge(edge));
        }
    }

    /// Turns the input method on with `state`, updates it, or turns it off with `None`.
    pub fn set_text_input(&mut self, state: Option<TextInputState>) {
        if self.text_input != state {
            self.text_input = state;
            self.text_input_changed = true;
        }
    }

    pub(crate) fn text_input(&self) -> Option<&TextInputState> {
        self.text_input.as_ref()
    }

    pub(crate) fn take_text_input_change(&mut self) -> bool {
        std::mem::take(&mut self.text_input_changed)
    }

    pub(crate) fn request_srgb(&mut self) {
        self.srgb_requested = true;
    }

    pub(crate) fn tag_color(&mut self, tag: impl FnOnce(&WlSurface) -> Option<SurfaceColorTag>) {
        if self.srgb_requested && self.color.is_none() {
            self.color = tag(self.shell.wl_surface());
            if self.color.is_some() {
                self.request_redraw();
            }
        }
    }

    pub(crate) fn take_cursor_change(&mut self) -> Option<CursorIcon> {
        std::mem::take(&mut self.cursor_changed).then_some(self.cursor)
    }

    pub(crate) fn record_button_press(&mut self, seat: WlSeat, serial: u32) {
        self.last_button_press = Some((seat, serial));
    }

    pub(crate) fn uses_fractional_scale(&self) -> bool {
        self.scaling.is_fractional()
    }

    /// Sets the compositor-rendered material, applied with the next frame.
    pub fn set_background_effect(&mut self, effect: &BackgroundEffect) -> Result<(), Error> {
        let surface = self
            .background_effect
            .as_ref()
            .ok_or(Error::MissingGlobal("crownos_background_effect_manager_v1"))?;
        surface.apply(effect);
        self.request_redraw();
        Ok(())
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        if let Some(inhibitor) = self.idle_inhibitor.take() {
            inhibitor.destroy();
        }
    }
}

impl WindowControl for Window {
    fn id(&self) -> WindowId {
        self.id
    }

    fn logical_size(&self) -> LogicalSize {
        self.logical_size
    }

    fn inner_size(&self) -> PhysicalSize {
        Window::inner_size(self)
    }

    fn scale_factor(&self) -> ScaleFactor {
        self.scale_factor
    }

    fn output_size(&self) -> Option<LogicalSize> {
        self.output_size
            .map(|(width, height)| LogicalSize::new(width, height))
    }

    fn is_configured(&self) -> bool {
        self.configured
    }

    fn request_redraw(&mut self) {
        Window::request_redraw(self);
    }

    fn set_title(&mut self, title: &str) {
        Window::set_title(self, title);
    }

    fn set_cursor(&mut self, icon: CursorIcon) {
        Window::set_cursor(self, icon);
    }

    fn set_material(&mut self, material: Option<&Material>) {
        let effect = material
            .map(|material| BackgroundEffect::from_material(material, None))
            .unwrap_or_default();
        let _ = self.set_background_effect(&effect);
    }

    fn begin_move(&mut self) {
        self.drag_window();
    }

    fn begin_resize(&mut self, edge: ResizeEdge) {
        self.drag_resize_window(edge);
    }

    fn minimize(&mut self) {
        Window::minimize(self);
    }

    fn toggle_maximize(&mut self) {
        if self.maximized {
            self.unmaximize();
        } else {
            self.maximize();
        }
    }

    fn is_floating(&self) -> bool {
        self.layer.is_none() && self.floating
    }

    fn set_text_input(&mut self, state: Option<TextInputState>) {
        Window::set_text_input(self, state);
    }

    fn set_layer_height(&mut self, height: LayerExtent) {
        let (Some(options), ShellSurface::Layer(layer)) = (&mut self.layer, &self.shell) else {
            return;
        };
        if options.height == height {
            return;
        }
        options.height = height;
        let fallback = (self.logical_size.width, self.logical_size.height);
        let (width, height) = options.requested_size(self.output_size, fallback);
        layer.set_size(width, height);
        self.shell.commit();
    }

    fn set_idle_inhibited(&mut self, inhibited: bool) {
        if inhibited == self.idle_inhibitor.is_some() {
            return;
        }
        match self.idle_inhibitor.take() {
            Some(inhibitor) => inhibitor.destroy(),
            None => {
                self.idle_inhibitor = self.idle_inhibit_manager.as_ref().map(|manager| {
                    manager.create_inhibitor(self.shell.wl_surface(), &self.queue_handle, NoEvents)
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wayland::EventLoopProxy;

    #[test]
    fn render_thread_handles_are_sendable() {
        fn assert_send<T: Send>() {}
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SurfaceHandle>();
        assert_send::<RenderSurface>();
        assert_send_sync::<EventLoopProxy>();
    }
}
