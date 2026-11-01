use geometry::Rect;
use reactive::ScopeId;
use windowing::AppIconPlacement;

use crate::{Cx, app_icon_events::AppIconListener};

/// An app icon change for the platform (see [`Cx::set_app_icon`]).
#[derive(Debug, Clone, PartialEq)]
pub enum AppIconRequest {
    /// Show the icon of an app at a placement, or move it there.
    Set(String, AppIconPlacement),
    /// Stop showing the icon of an app.
    Remove(String),
    /// The user activated the icon of an app.
    Activate(String),
}

/// The app icons the window shows, the changes the platform has not heard of yet, and who wants
/// to hear about the apps behind them.
#[derive(Default)]
pub(crate) struct AppIcons {
    shown: Vec<(String, AppIconPlacement)>,
    requests: Vec<AppIconRequest>,
    pub(crate) listeners: Vec<(ScopeId, AppIconListener)>,
}

impl AppIcons {
    pub(crate) fn take_requests(&mut self) -> Vec<AppIconRequest> {
        std::mem::take(&mut self.requests)
    }

    fn is_shown(&self, app_id: &str) -> bool {
        self.shown.iter().any(|(shown, _)| shown == app_id)
    }
}

impl Cx {
    /// Tells the compositor that the window shows the icon of the application `app_id` at `rect`
    /// with `corner_radius`, where it rests when the window is fully shown: the app's windows
    /// open out of it and minimize into it. Calling it again moves the icon; an unchanged
    /// placement sends nothing.
    pub fn set_app_icon(&mut self, app_id: &str, rect: Rect, corner_radius: f32) {
        let placement = AppIconPlacement {
            rect,
            corner_radius,
        };
        let icons = &mut self.app_icons;
        match icons.shown.iter_mut().find(|(shown, _)| shown == app_id) {
            Some((_, current)) if *current == placement => return,
            Some((_, current)) => *current = placement,
            None => icons.shown.push((app_id.to_owned(), placement)),
        }
        icons
            .requests
            .push(AppIconRequest::Set(app_id.to_owned(), placement));
    }

    /// Stops showing the icon of `app_id` (see [`Cx::set_app_icon`]).
    pub fn remove_app_icon(&mut self, app_id: &str) {
        let icons = &mut self.app_icons;
        let before = icons.shown.len();
        icons.shown.retain(|(shown, _)| shown != app_id);
        if icons.shown.len() != before {
            icons
                .requests
                .push(AppIconRequest::Remove(app_id.to_owned()));
        }
    }

    /// Hands a click on the icon of `app_id` to the compositor, which restores and focuses the
    /// app's windows or, when it has none, asks [`Cx::on_launch_app`] listeners to start it.
    /// Returns `false` when the platform has no app icons or the window does not show this one,
    /// so the caller starts the app itself.
    #[must_use]
    pub fn activate_app_icon(&mut self, app_id: &str) -> bool {
        let supported = self
            .capabilities
            .with_untracked(self, |capabilities| capabilities.app_icons);
        if !supported || !self.app_icons.is_shown(app_id) {
            return false;
        }
        self.app_icons
            .requests
            .push(AppIconRequest::Activate(app_id.to_owned()));
        true
    }
}
