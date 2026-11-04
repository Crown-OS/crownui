use reactive::ScopeId;

use crate::{Cx, Event};

type CloseListener = Box<dyn FnMut(&mut Cx, &mut Event<()>)>;

/// The window's `on_close_requested` listeners and whether it should close.
#[derive(Default)]
pub(crate) struct CloseRequests {
    listeners: Vec<(ScopeId, CloseListener)>,
    approved: bool,
}

impl Cx {
    /// Calls `listener` when the user or the compositor asks to close the window, until the
    /// calling component unmounts. `event.prevent_default()` keeps the window open, e.g. while a
    /// long operation runs or to ask the user first; [`Cx::close_window`] closes it afterwards.
    /// Without listeners the window closes as soon as it is asked to.
    pub fn on_close_requested(&mut self, listener: impl FnMut(&mut Cx, &mut Event<()>) + 'static) {
        let scope = self.current_scope();
        self.close_requests
            .listeners
            .push((scope, Box::new(listener)));
    }

    /// Closes the window without asking the `on_close_requested` listeners.
    pub fn close_window(&mut self) {
        self.close_requests.approved = true;
    }

    /// Asks every listener whether the window may close; it closes unless one prevents it.
    pub(crate) fn ask_to_close(&mut self) {
        let mut listeners = std::mem::take(&mut self.close_requests.listeners);
        listeners.retain(|(scope, _)| self.is_scope_alive(*scope));
        let mut event = Event::new((), false);
        for (scope, listener) in &mut listeners {
            let previous = self.set_current_scope(*scope);
            listener(self, &mut event);
            self.set_current_scope(previous);
        }
        listeners.append(&mut self.close_requests.listeners);
        self.close_requests.listeners = listeners;
        self.close_requests.approved |= !event.is_default_prevented();
        self.apply_pending();
    }

    pub(crate) fn take_close_approval(&mut self) -> bool {
        std::mem::take(&mut self.close_requests.approved)
    }
}
