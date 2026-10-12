use crate::{
    AppIconPlacement, Capabilities, ControlFlow, Platform, WindowAttributes, WindowId,
    wayland::{CrownEvent, Error, OutputEvent, RenderSurface, Window, event_loop::WaylandPlatform},
};

/// Wayland-specific events beyond [`WindowEvent`](crate::WindowEvent).
#[derive(Debug)]
pub enum WaylandEvent {
    /// A monitor appeared, changed or vanished.
    Output(OutputEvent),
    /// A CrownOS protocol reported something.
    Crown(CrownEvent),
}

impl Platform for WaylandPlatform {
    type Window = Window;
    type Surface = RenderSurface;
    type Event = WaylandEvent;
    type Error = Error;

    fn create_window(&mut self, attributes: WindowAttributes) -> Result<WindowId, Error> {
        Ok(WaylandPlatform::create_window(self, attributes))
    }

    fn window(&self, id: WindowId) -> Option<&Window> {
        WaylandPlatform::window(self, id)
    }

    fn window_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        WaylandPlatform::window_mut(self, id)
    }

    fn close_window(&mut self, id: WindowId) {
        WaylandPlatform::close_window(self, id);
    }

    fn surface(&self, id: WindowId) -> Option<RenderSurface> {
        WaylandPlatform::window(self, id).map(Window::render_surface)
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            materials: self.crown.material.is_some(),
            fractional_scale: self.scaling.is_some(),
            touchpad_gestures: self.standard.pointer_gestures.is_some(),
            text_input: self.standard.text_input.is_some(),
            popups: true,
            presentation_time: self.standard.presentation.is_some(),
            visibility_hints: self.crown.visibility.is_some(),
            surface_animation: self.crown.surface_animation.is_some(),
            layer_shell: self.standard.layer_shell.is_some(),
            idle_inhibit: self.standard.idle_inhibit.is_some(),
            app_icons: self.crown.app_icons.is_some(),
        }
    }

    fn set_control_flow(&mut self, control_flow: ControlFlow) {
        WaylandPlatform::set_control_flow(self, control_flow);
    }

    fn exit(&mut self) {
        WaylandPlatform::exit(self);
    }

    fn request_agent_access(&mut self, app_id: &str, reason: &str) -> bool {
        match self.create_agent_access_request(app_id, reason) {
            Ok(request) => {
                self.agent_access_request = Some(request);
                true
            }
            Err(_) => false,
        }
    }

    fn waker(&self) -> crate::Waker {
        let proxy = self.create_proxy();
        Box::new(move || proxy.wake_up())
    }

    fn set_clipboard_text(&mut self, text: String) {
        WaylandPlatform::set_clipboard_text(self, text);
    }

    fn request_clipboard_text(&mut self) {
        WaylandPlatform::request_clipboard_text(self);
    }

    fn start_drag(&mut self, window: WindowId, source: crate::DragSource) {
        WaylandPlatform::start_drag(self, window, source);
    }

    fn accept_drag(&mut self, mime_type: Option<&str>) {
        WaylandPlatform::accept_drag(self, mime_type);
    }

    fn set_app_icon(&mut self, window: WindowId, app_id: &str, placement: AppIconPlacement) {
        WaylandPlatform::set_app_icon(self, window, app_id, placement);
    }

    fn remove_app_icon(&mut self, window: WindowId, app_id: &str) {
        WaylandPlatform::remove_app_icon(self, window, app_id);
    }

    fn activate_app_icon(&mut self, window: WindowId, app_id: &str) {
        WaylandPlatform::activate_app_icon(self, window, app_id);
    }
}
