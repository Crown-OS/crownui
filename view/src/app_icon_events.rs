use windowing::AppWindows;

use crate::Cx;

type LaunchListener = Box<dyn FnMut(&mut Cx, &str)>;
type WindowsListener = Box<dyn FnMut(&mut Cx, &AppWindows)>;
type LiftListener = Box<dyn FnMut(&mut Cx, &str, bool)>;

/// Something the platform told the window about the apps behind its icons.
#[derive(Debug, Clone, Copy)]
pub(crate) enum AppIconEvent<'a> {
    /// Start the app: its icon was activated while it had no windows.
    Launch(&'a str),
    /// The app's windows changed.
    Windows(&'a AppWindows),
    /// The platform started (`true`) or stopped drawing the app's icon.
    Lifted(&'a str, bool),
}

/// A component's interest in one kind of [`AppIconEvent`].
pub(crate) enum AppIconListener {
    Launch(LaunchListener),
    Windows(WindowsListener),
    Lifted(LiftListener),
}

impl AppIconListener {
    fn hear(&mut self, cx: &mut Cx, event: AppIconEvent) {
        match (self, event) {
            (Self::Launch(listener), AppIconEvent::Launch(app_id)) => listener(cx, app_id),
            (Self::Windows(listener), AppIconEvent::Windows(windows)) => listener(cx, windows),
            (Self::Lifted(listener), AppIconEvent::Lifted(app_id, lifted)) => {
                listener(cx, app_id, lifted)
            }
            _ => {}
        }
    }
}

impl Cx {
    /// Calls `listener` with the app id whenever the compositor asks to start an app whose icon
    /// was activated, until the calling component unmounts.
    pub fn on_launch_app(&mut self, listener: impl FnMut(&mut Cx, &str) + 'static) {
        self.listen_to_app_icons(AppIconListener::Launch(Box::new(listener)));
    }

    /// Calls `listener` whenever an app's windows change: how many it has and whether one is
    /// focused, starting with every app already running, until the calling component unmounts.
    /// A count of 0 means the app has no window left.
    pub fn on_app_windows(&mut self, listener: impl FnMut(&mut Cx, &AppWindows) + 'static) {
        self.listen_to_app_icons(AppIconListener::Windows(Box::new(listener)));
    }

    /// Calls `listener` with the app id and `true` when the compositor starts drawing the app's
    /// icon as part of a window's motion, and `false` when it stops, until the calling component
    /// unmounts. In between, the icon keeps its place but should not be drawn.
    pub fn on_app_icon_lifted(&mut self, listener: impl FnMut(&mut Cx, &str, bool) + 'static) {
        self.listen_to_app_icons(AppIconListener::Lifted(Box::new(listener)));
    }

    fn listen_to_app_icons(&mut self, listener: AppIconListener) {
        let scope = self.current_scope();
        self.app_icons.listeners.push((scope, listener));
    }

    pub(crate) fn notify_app_icons(&mut self, event: AppIconEvent) {
        let mut listeners = std::mem::take(&mut self.app_icons.listeners);
        listeners.retain(|(scope, _)| self.is_scope_alive(*scope));
        for (scope, listener) in &mut listeners {
            let previous = self.set_current_scope(*scope);
            listener.hear(self, event);
            self.set_current_scope(previous);
        }
        listeners.append(&mut self.app_icons.listeners);
        self.app_icons.listeners = listeners;
        self.apply_pending();
    }
}
