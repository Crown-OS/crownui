use crate::Cx;

/// A window-manager operation a handler asked for, carried out by the platform after the event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowRequest {
    /// Start an interactive move with the pointer press being handled.
    Move,
    /// Hide the window until the user brings it back.
    Minimize,
    /// Maximize the window, or restore it when it already is.
    ToggleMaximize,
}

impl Cx {
    /// Lets the user drag the window by the element being pressed; call it from a pointer-down
    /// handler, as a client-drawn title bar does.
    pub fn begin_window_move(&mut self) {
        self.window_requests.push(WindowRequest::Move);
    }

    /// Minimizes the window.
    pub fn minimize_window(&mut self) {
        self.window_requests.push(WindowRequest::Minimize);
    }

    /// Maximizes the window, or restores it when it already is.
    pub fn toggle_maximize_window(&mut self) {
        self.window_requests.push(WindowRequest::ToggleMaximize);
    }

    /// Asks to close the window as its close button does: the `on_close_requested` listeners
    /// may still keep it open (see [`Cx::close_window`] to skip them).
    pub fn request_close(&mut self) {
        self.ask_to_close();
    }
}
