use geometry::Point;
use sctk::{
    data_device_manager::{
        data_device::DataDeviceHandler,
        data_offer::{DataOfferHandler, DragOffer},
    },
    reexports::client::{
        Connection, QueueHandle,
        protocol::{
            wl_data_device::WlDataDevice, wl_data_device_manager::DndAction as WaylandAction,
            wl_surface::WlSurface,
        },
    },
};

use super::dnd_action::{from_wayland_action, from_wayland_actions, to_wayland_action};
use crate::{
    DndAction, DropAcceptance, DroppedData, WindowEvent, WindowId,
    wayland::{event_loop::WaylandPlatform, window::surface_window_id},
};

/// A drag over one of this client's windows, from enter until it leaves or is dropped.
pub(crate) struct DropTarget {
    window: WindowId,
    offer: DragOffer,
    position: Point,
    accepted: Option<DropAcceptance>,
    selected: Option<DndAction>,
}

impl WaylandPlatform {
    fn current_drag_offer(&self, device: &WlDataDevice) -> Option<DragOffer> {
        self.seats
            .iter()
            .filter_map(|devices| devices.data_device.as_ref())
            .find(|data_device| data_device.inner() == device)
            .and_then(|data_device| data_device.data().drag_offer())
    }

    /// Accepts the drag over a window, or refuses it with `None`. Accepting offers copy and
    /// move (and asking, when that is preferred) with the acceptance's action preferred; the
    /// compositor settles on one the source allows.
    pub fn accept_drag(&mut self, acceptance: Option<&DropAcceptance>) {
        let Some(target) = self.drop_target.as_mut() else {
            return;
        };
        if target.accepted.as_ref() == acceptance {
            return;
        }
        let (actions, preferred) = match acceptance {
            Some(acceptance) => {
                let preferred = to_wayland_action(acceptance.action);
                (
                    WaylandAction::Copy | WaylandAction::Move | preferred,
                    preferred,
                )
            }
            None => (WaylandAction::empty(), WaylandAction::empty()),
        };
        target.offer.accept_mime_type(
            target.offer.serial,
            acceptance.map(|acceptance| acceptance.mime_type.clone()),
        );
        target.offer.set_actions(actions, preferred);
        target.accepted = acceptance.cloned();
    }

    fn finish_drop(&mut self, target: DropTarget) {
        let DropTarget {
            window,
            offer,
            position,
            accepted,
            selected,
        } = target;
        let pipe = accepted
            .as_ref()
            .and_then(|accepted| offer.receive(accepted.mime_type.clone()).ok());
        let (Some(accepted), Some(pipe)) = (accepted, pipe) else {
            offer.destroy();
            self.push_window_event(window, WindowEvent::DragLeave);
            return;
        };
        let action = selected.unwrap_or(accepted.action);
        if action == DndAction::Ask {
            offer.set_actions(WaylandAction::Copy, WaylandAction::Copy);
        }
        let reader_offer = offer.clone();
        let reading = self.read_pipe(pipe, move |state, data| {
            reader_offer.finish();
            reader_offer.destroy();
            state.push_window_event(
                window,
                WindowEvent::Drop(DroppedData {
                    position,
                    mime_type: accepted.mime_type,
                    data,
                    action,
                }),
            );
        });
        if !reading {
            offer.destroy();
            self.push_window_event(window, WindowEvent::DragLeave);
        }
    }
}

impl DataDeviceHandler for WaylandPlatform {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        device: &WlDataDevice,
        x: f64,
        y: f64,
        surface: &WlSurface,
    ) {
        let window = surface_window_id(surface);
        let Some(offer) = self
            .windows
            .contains_key(&window)
            .then(|| self.current_drag_offer(device))
            .flatten()
        else {
            return;
        };
        let position = Point::new(x as f32, y as f32);
        let mime_types = offer.with_mime_types(<[String]>::to_vec);
        let actions = from_wayland_actions(offer.source_actions);
        self.drop_target = Some(DropTarget {
            window,
            offer,
            position,
            accepted: None,
            selected: None,
        });
        self.push_window_event(
            window,
            WindowEvent::DragEnter(crate::DragOffer {
                position,
                mime_types,
                actions,
            }),
        );
    }

    fn leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice) {
        if let Some(target) = self.drop_target.take() {
            self.push_window_event(target.window, WindowEvent::DragLeave);
        }
    }

    fn motion(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice, x: f64, y: f64) {
        let Some(target) = self.drop_target.as_mut() else {
            return;
        };
        target.position = Point::new(x as f32, y as f32);
        let (window, position) = (target.window, target.position);
        self.push_window_event(window, WindowEvent::DragOver(position));
    }

    fn selection(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice) {}

    fn drop_performed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice) {
        if let Some(target) = self.drop_target.take() {
            self.finish_drop(target);
        }
    }
}

impl DataOfferHandler for WaylandPlatform {
    fn source_actions(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &mut DragOffer,
        _: WaylandAction,
    ) {
    }

    fn selected_action(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        offer: &mut DragOffer,
        action: WaylandAction,
    ) {
        if let Some(target) = self
            .drop_target
            .as_mut()
            .filter(|target| target.offer.inner() == offer.inner())
        {
            target.selected = from_wayland_action(action);
        }
    }
}
