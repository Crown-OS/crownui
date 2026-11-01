use crownos_protocols::app_icons::v1::client::{
    crownos_app_icon_manager_v1::{self, CrownosAppIconManagerV1},
    crownos_app_icon_v1::{self, CrownosAppIconV1},
};
use sctk::{
    dispatch2::Dispatch2,
    reexports::client::{Connection, QueueHandle, globals::GlobalList},
};

use super::CrownEvent;
use crate::{
    AppIconPlacement, AppWindows, WindowEvent, WindowId,
    wayland::event_loop::{QueuedEvent, WaylandPlatform},
};

/// The icons this client shows, each on one window for one application, and the applications
/// the compositor reported windows for.
pub(crate) struct AppIconsGlobal {
    manager: CrownosAppIconManagerV1,
    icons: Vec<AppIcon>,
    running: Vec<AppWindows>,
}

struct AppIcon {
    window: WindowId,
    app_id: String,
    proxy: CrownosAppIconV1,
}

impl AppIcon {
    fn is(&self, window: WindowId, app_id: &str) -> bool {
        self.window == window && self.app_id == app_id
    }
}

impl Drop for AppIcon {
    fn drop(&mut self) {
        self.proxy.destroy();
    }
}

impl AppIconsGlobal {
    pub(crate) fn bind(globals: &GlobalList, qh: &QueueHandle<WaylandPlatform>) -> Option<Self> {
        Some(Self {
            manager: globals.bind(qh, 1..=2, AppIconManagerData).ok()?,
            icons: Vec::new(),
            running: Vec::new(),
        })
    }

    fn icon(&self, window: WindowId, app_id: &str) -> Option<&AppIcon> {
        self.icons.iter().find(|icon| icon.is(window, app_id))
    }

    fn remove(&mut self, window: WindowId, app_id: &str) {
        self.icons.retain(|icon| !icon.is(window, app_id));
    }

    /// Destroys the icons on `window`, before its surface goes.
    pub(crate) fn forget_window(&mut self, window: WindowId) {
        self.icons.retain(|icon| icon.window != window);
    }
}

/// Records `update` in `running`, dropping the application once it has no window left.
fn remember(running: &mut Vec<AppWindows>, update: &AppWindows) {
    let known = running
        .iter()
        .position(|app| app.app_id.eq_ignore_ascii_case(&update.app_id));
    match (known, update.count) {
        (Some(index), 0) => {
            running.remove(index);
        }
        (Some(index), _) => running[index].clone_from(update),
        (None, 0) => {}
        (None, _) => running.push(update.clone()),
    }
}

/// The rectangle in whole logical pixels, or `None` when the compositor would reject it.
fn wire_rectangle(placement: AppIconPlacement) -> Option<(i32, i32, i32, i32, f64)> {
    let rect = placement.rect;
    let width = rect.size.width.round() as i32;
    let height = rect.size.height.round() as i32;
    (width > 0 && height > 0).then(|| {
        (
            rect.origin.x.round() as i32,
            rect.origin.y.round() as i32,
            width,
            height,
            f64::from(placement.corner_radius.max(0.0)),
        )
    })
}

impl WaylandPlatform {
    /// Shows the icon of `app_id` on `window` at `placement`, or moves it there. Does nothing on
    /// a compositor without `crownos_app_icon_manager_v1`, or for an empty rectangle.
    pub fn set_app_icon(&mut self, window: WindowId, app_id: &str, placement: AppIconPlacement) {
        let Some((x, y, width, height, corner_radius)) = wire_rectangle(placement) else {
            return;
        };
        let Some(surface) = self
            .windows
            .get(&window)
            .map(|window| window.wl_surface().clone())
        else {
            return;
        };
        let Some(icons) = self.crown.app_icons.as_mut() else {
            return;
        };
        if icons.icon(window, app_id).is_none() {
            let proxy = icons.manager.get_app_icon(
                &surface,
                app_id.to_owned(),
                &self.queue_handle,
                AppIconData {
                    window,
                    app_id: app_id.to_owned(),
                },
            );
            icons.icons.push(AppIcon {
                window,
                app_id: app_id.to_owned(),
                proxy,
            });
        }
        if let Some(icon) = icons.icon(window, app_id) {
            icon.proxy.set_rectangle(x, y, width, height, corner_radius);
        }
    }

    /// Destroys the icon of `app_id` on `window`.
    pub fn remove_app_icon(&mut self, window: WindowId, app_id: &str) {
        if let Some(icons) = self.crown.app_icons.as_mut() {
            icons.remove(window, app_id);
        }
    }

    /// Tells the compositor the user activated the icon of `app_id` on `window`; when the app has
    /// no windows, [`CrownEvent::AppIconLaunch`] follows.
    pub fn activate_app_icon(&mut self, window: WindowId, app_id: &str) {
        if let Some(icon) = self
            .crown
            .app_icons
            .as_ref()
            .and_then(|icons| icons.icon(window, app_id))
        {
            icon.proxy.activate();
        }
    }

    /// Tells `window` which applications have windows, when it opens after the compositor did.
    pub(crate) fn replay_app_windows(&mut self, window: WindowId) {
        let Some(icons) = self.crown.app_icons.as_ref() else {
            return;
        };
        self.pending_events.extend(
            icons
                .running
                .iter()
                .map(|app| QueuedEvent::Window(window, WindowEvent::AppWindows(app.clone()))),
        );
    }

    fn app_windows_changed(&mut self, update: AppWindows) {
        if let Some(icons) = self.crown.app_icons.as_mut() {
            remember(&mut icons.running, &update);
        }
        self.pending_events.extend(
            self.windows.keys().map(|&window| {
                QueuedEvent::Window(window, WindowEvent::AppWindows(update.clone()))
            }),
        );
    }
}

pub(crate) struct AppIconManagerData;

impl Dispatch2<CrownosAppIconManagerV1, WaylandPlatform> for AppIconManagerData {
    fn event(
        &self,
        state: &mut WaylandPlatform,
        _: &CrownosAppIconManagerV1,
        event: crownos_app_icon_manager_v1::Event,
        _: &Connection,
        _: &QueueHandle<WaylandPlatform>,
    ) {
        if let crownos_app_icon_manager_v1::Event::AppWindows {
            app_id,
            count,
            focused,
        } = event
        {
            state.app_windows_changed(AppWindows {
                app_id,
                count,
                focused: focused != 0,
            });
        }
    }
}

pub(crate) struct AppIconData {
    window: WindowId,
    app_id: String,
}

impl Dispatch2<CrownosAppIconV1, WaylandPlatform> for AppIconData {
    fn event(
        &self,
        state: &mut WaylandPlatform,
        _: &CrownosAppIconV1,
        event: crownos_app_icon_v1::Event,
        _: &Connection,
        _: &QueueHandle<WaylandPlatform>,
    ) {
        let lifted = match event {
            crownos_app_icon_v1::Event::Launch => {
                state
                    .pending_events
                    .push(QueuedEvent::Crown(CrownEvent::AppIconLaunch {
                        window: self.window,
                        app_id: self.app_id.clone(),
                    }));
                return;
            }
            crownos_app_icon_v1::Event::Lifted => true,
            crownos_app_icon_v1::Event::Landed => false,
            _ => return,
        };
        state.push_window_event(
            self.window,
            WindowEvent::AppIconLifted {
                app_id: self.app_id.clone(),
                lifted,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use geometry::Rect;

    use super::*;

    fn app(app_id: &str, count: u32, focused: bool) -> AppWindows {
        AppWindows {
            app_id: app_id.to_owned(),
            count,
            focused,
        }
    }

    #[test]
    fn the_latest_report_of_an_app_replaces_the_previous_one() {
        let mut running = Vec::new();
        remember(&mut running, &app("org.example.Editor", 1, false));
        remember(&mut running, &app("org.example.Files", 2, false));
        remember(&mut running, &app("ORG.example.editor", 3, true));
        assert_eq!(
            running,
            [
                app("ORG.example.editor", 3, true),
                app("org.example.Files", 2, false)
            ]
        );
    }

    #[test]
    fn an_app_without_windows_is_forgotten() {
        let mut running = vec![app("org.example.Editor", 1, true)];
        remember(&mut running, &app("org.example.Files", 0, false));
        remember(&mut running, &app("org.example.editor", 0, false));
        assert!(running.is_empty());
    }

    #[test]
    fn a_rectangle_is_sent_in_whole_logical_pixels() {
        let placement = AppIconPlacement {
            rect: Rect::from_xywh(10.4, 20.6, 51.6, 52.0),
            corner_radius: 11.44,
        };
        assert_eq!(
            wire_rectangle(placement),
            Some((10, 21, 52, 52, f64::from(11.44_f32)))
        );
    }

    #[test]
    fn a_rectangle_the_compositor_would_reject_is_not_sent() {
        let empty = AppIconPlacement {
            rect: Rect::from_xywh(0.0, 0.0, 0.3, 52.0),
            corner_radius: 4.0,
        };
        assert_eq!(wire_rectangle(empty), None);
        let negative_radius = AppIconPlacement {
            rect: Rect::from_xywh(0.0, 0.0, 52.0, 52.0),
            corner_radius: -1.0,
        };
        assert_eq!(
            wire_rectangle(negative_radius).map(|wire| wire.4),
            Some(0.0)
        );
    }
}
