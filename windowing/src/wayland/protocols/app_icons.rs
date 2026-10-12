use crownos_protocols::app_icons::v1::client::{
    crownos_app_icon_manager_v1::CrownosAppIconManagerV1,
    crownos_app_icon_v1::{self, CrownosAppIconV1},
};
use sctk::{
    dispatch2::Dispatch2,
    reexports::client::{Connection, QueueHandle, globals::GlobalList},
};

use super::CrownEvent;
use crate::{
    AppIconPlacement, WindowId,
    wayland::{
        event_loop::{QueuedEvent, WaylandPlatform},
        util::no_events::NoEvents,
    },
};

/// The icons this client shows, each on one window for one application.
pub(crate) struct AppIconsGlobal {
    manager: CrownosAppIconManagerV1,
    icons: Vec<AppIcon>,
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
            manager: globals.bind(qh, 1..=1, NoEvents).ok()?,
            icons: Vec::new(),
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
        if let crownos_app_icon_v1::Event::Launch = event {
            state
                .pending_events
                .push(QueuedEvent::Crown(CrownEvent::AppIconLaunch {
                    window: self.window,
                    app_id: self.app_id.clone(),
                }));
        }
    }
}

#[cfg(test)]
mod tests {
    use geometry::Rect;

    use super::*;

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
