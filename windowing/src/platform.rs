use std::time::Instant;

use crate::{
    AppIconPlacement, CursorIcon, DragSource, Material, PlatformSurface, TextInputState,
    WindowAttributes, WindowEvent, WindowId,
    dpi::{LogicalSize, PhysicalSize, ScaleFactor},
};

/// What a platform can do, so the runtime picks fallbacks instead of failing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Capabilities {
    /// The platform renders [`Material`] backgrounds (otherwise use in-app backdrop blur).
    pub materials: bool,
    /// Non-integer scale factors reach the app unrounded.
    pub fractional_scale: bool,
    /// Multi-finger touchpad gestures are reported.
    pub touchpad_gestures: bool,
    /// Input-method text input is available.
    pub text_input: bool,
    /// Popups may extend outside their parent window.
    pub popups: bool,
    /// Frame callbacks carry presentation timestamps.
    pub presentation_time: bool,
    /// The platform reports whether each window is visible, occluded or hidden.
    pub visibility_hints: bool,
    /// The compositor animates subsurface transforms and opacity itself.
    pub surface_animation: bool,
    /// Windows can be layer surfaces (panels, docks, overlays).
    pub layer_shell: bool,
    /// A window can keep the system from idling (screen blanking, sleep) while it is visible.
    pub idle_inhibit: bool,
    /// A launcher can tell the compositor where it shows application icons, so windows open
    /// out of them and minimize into them.
    pub app_icons: bool,
}

/// When the event loop should wake up again.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ControlFlow {
    /// Sleep until an event arrives.
    #[default]
    Wait,
    /// Never sleep.
    Poll,
    /// Sleep until an event arrives or the deadline passes.
    WaitUntil(Instant),
}

/// Operations on one window.
pub trait WindowControl {
    /// This window's id.
    fn id(&self) -> WindowId;
    /// Content size in logical pixels.
    fn logical_size(&self) -> LogicalSize;
    /// Drawable size in device pixels.
    fn inner_size(&self) -> PhysicalSize;
    /// Current scale factor.
    fn scale_factor(&self) -> ScaleFactor;
    /// Asks for a [`WindowEvent::RedrawRequested`] when the platform is ready.
    fn request_redraw(&mut self);
    /// Sets the title.
    fn set_title(&mut self, title: &str);
    /// Sets the cursor shown while the pointer is over the window.
    fn set_cursor(&mut self, icon: CursorIcon);
    /// Sets or clears the platform-rendered background.
    fn set_material(&mut self, material: Option<&Material>);
    /// Starts an interactive move; call it while handling a pointer down.
    fn begin_move(&mut self);
    /// Turns the input method on for this window with `state`, updates it, or turns it off
    /// with `None`; composition arrives as [`WindowEvent::TextInput`].
    fn set_text_input(&mut self, state: Option<TextInputState>);
    /// Keeps the system from idling while this window is visible, or lets it idle again.
    fn set_idle_inhibited(&mut self, inhibited: bool);
}

/// A windowing backend; the runtime is generic over it.
pub trait Platform: Sized + 'static {
    /// The backend's window type.
    type Window: WindowControl;
    /// The render-thread surface handle.
    type Surface: PlatformSurface;
    /// Backend-specific events beyond [`WindowEvent`].
    type Event;
    /// Backend errors.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Creates a window.
    fn create_window(&mut self, attributes: WindowAttributes) -> Result<WindowId, Self::Error>;
    /// Looks up a window.
    fn window(&self, id: WindowId) -> Option<&Self::Window>;
    /// Looks up a window mutably.
    fn window_mut(&mut self, id: WindowId) -> Option<&mut Self::Window>;
    /// Closes a window; its surfaces stop presenting.
    fn close_window(&mut self, id: WindowId);
    /// A surface for presenting `id` from another thread.
    fn surface(&self, id: WindowId) -> Option<Self::Surface>;
    /// What this platform supports.
    fn capabilities(&self) -> Capabilities;
    /// Sets when the loop wakes up next.
    fn set_control_flow(&mut self, control_flow: ControlFlow);
    /// Asks the event loop to return.
    fn exit(&mut self);
    /// Asks the platform (and through it the user) to let AI agents read and drive this app.
    /// Returns `false` when the platform has no such gate; otherwise the answer arrives through
    /// [`ApplicationHandler::agent_access`].
    fn request_agent_access(&mut self, app_id: &str, reason: &str) -> bool {
        let _ = (app_id, reason);
        false
    }
    /// A thread-safe callback that wakes the event loop and triggers
    /// [`ApplicationHandler::proxy_wake_up`].
    fn waker(&self) -> Waker;
    /// Offers `text` as the clipboard contents.
    fn set_clipboard_text(&mut self, text: String);
    /// Asks for the clipboard's text; it arrives as [`WindowEvent::ClipboardText`] at the
    /// focused window, empty when the clipboard holds no text.
    fn request_clipboard_text(&mut self);
    /// Starts dragging `source` out of `window`; call it while a pointer button is held. The end
    /// arrives as [`WindowEvent::DragEnded`] at `window`.
    fn start_drag(&mut self, window: WindowId, source: DragSource) {
        let _ = (window, source);
    }
    /// Accepts the drag over the window as `mime_type`, or refuses it with `None`; answer each
    /// [`WindowEvent::DragEnter`] and [`WindowEvent::DragOver`].
    fn accept_drag(&mut self, mime_type: Option<&str>) {
        let _ = mime_type;
    }
    /// Tells the platform that `window` shows the icon of the application `app_id` at
    /// `placement`, or moves it there; see [`Capabilities::app_icons`].
    fn set_app_icon(&mut self, window: WindowId, app_id: &str, placement: AppIconPlacement) {
        let _ = (window, app_id, placement);
    }
    /// `window` no longer shows the icon of `app_id`.
    fn remove_app_icon(&mut self, window: WindowId, app_id: &str) {
        let _ = (window, app_id);
    }
    /// The user activated the icon of `app_id` on `window`. The platform restores and focuses the
    /// application's windows, or answers [`WindowEvent::LaunchApp`] when it has none.
    fn activate_app_icon(&mut self, window: WindowId, app_id: &str) {
        let _ = (window, app_id);
    }
}

/// Wakes a platform's event loop from any thread.
pub type Waker = Box<dyn Fn() + Send + Sync>;

/// Callbacks a backend's event loop drives.
pub trait ApplicationHandler<P: Platform> {
    /// Called once when windows can be created.
    fn resumed(&mut self, platform: &mut P);

    /// A window reported an event.
    fn window_event(&mut self, platform: &mut P, window_id: WindowId, event: WindowEvent);

    /// A backend-specific event arrived.
    fn platform_event(&mut self, _platform: &mut P, _event: P::Event) {}

    /// The platform answered [`Platform::request_agent_access`].
    fn agent_access(&mut self, _platform: &mut P, _decision: crate::AgentAccess) {}

    /// Another thread woke the loop through its proxy.
    fn proxy_wake_up(&mut self, _platform: &mut P) {}

    /// Every queued event was delivered; the loop is about to sleep.
    fn about_to_wait(&mut self, _platform: &mut P) {}
}
