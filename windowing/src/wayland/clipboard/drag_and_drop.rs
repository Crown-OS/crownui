use geometry::Point;
use sctk::{
    data_device_manager::{
        data_device::DataDeviceHandler,
        data_offer::{DataOfferHandler, DragOffer},
    },
    reexports::client::{
        Connection, QueueHandle,
        protocol::{
            wl_data_device::WlDataDevice, wl_data_device_manager::DndAction, wl_surface::WlSurface,
        },
    },
};

use crate::{
    DroppedData, WindowEvent, WindowId,
    wayland::{event_loop::WaylandPlatform, window::surface_window_id},
};

/// A drag over one of this client's windows, from enter until it leaves or is dropped.
pub(crate) struct DropTarget {
    window: WindowId,
    offer: DragOffer,
    position: Point,
    accepted: Option<String>,
}

impl WaylandPlatform {
    fn current_drag_offer(&self, device: &WlDataDevice) -> Option<DragOffer> {
        self.seats
            .iter()
            .filter_map(|devices| devices.data_device.as_ref())
            .find(|data_device| data_device.inner() == device)
            .and_then(|data_device| data_device.data().drag_offer())
    }

    /// Accepts the drag over a window as `mime_type`, or refuses it with `None`. Accepting
    /// offers copy and move, preferring copy, so the source keeps what it dragged.
    pub fn accept_drag(&mut self, mime_type: Option<&str>) {
        let Some(target) = self.drop_target.as_mut() else {
            return;
        };
        if target.accepted.as_deref() == mime_type {
            return;
        }
        let (actions, preferred) = match mime_type {
            Some(_) => (DndAction::Copy | DndAction::Move, DndAction::Copy),
            None => (DndAction::empty(), DndAction::empty()),
        };
        target
            .offer
            .accept_mime_type(target.offer.serial, mime_type.map(str::to_owned));
        target.offer.set_actions(actions, preferred);
        target.accepted = mime_type.map(str::to_owned);
    }

    fn finish_drop(&mut self, target: DropTarget) {
        let DropTarget {
            window,
            offer,
            position,
            accepted,
        } = target;
        let pipe = accepted
            .as_ref()
            .and_then(|mime_type| offer.receive(mime_type.clone()).ok());
        let (Some(mime_type), Some(pipe)) = (accepted, pipe) else {
            offer.destroy();
            self.push_window_event(window, WindowEvent::DragLeave);
            return;
        };
        let reader_offer = offer.clone();
        let reading = self.read_pipe(pipe, move |state, data| {
            reader_offer.finish();
            reader_offer.destroy();
            state.push_window_event(
                window,
                WindowEvent::Drop(DroppedData {
                    position,
                    mime_type,
                    data,
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
        self.drop_target = Some(DropTarget {
            window,
            offer,
            position,
            accepted: None,
        });
        self.push_window_event(
            window,
            WindowEvent::DragEnter(crate::DragOffer {
                position,
                mime_types,
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
        _: DndAction,
    ) {
    }

    fn selected_action(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &mut DragOffer,
        _: DndAction,
    ) {
    }
}
