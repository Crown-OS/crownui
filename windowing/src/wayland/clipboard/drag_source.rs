use sctk::{
    data_device_manager::data_source::DragSource as WaylandDragSource,
    reexports::client::protocol::{
        wl_data_device_manager::DndAction, wl_data_source::WlDataSource,
    },
};

use super::drag_icon::DragIconSurface;
use crate::{DragOutcome, DragSource, WindowEvent, WindowId, wayland::event_loop::WaylandPlatform};

/// A drag this client started, alive until the compositor reports how it ended.
pub(crate) struct ActiveDrag {
    window: WindowId,
    source: WaylandDragSource,
    items: Vec<(String, Vec<u8>)>,
    _icon: Option<DragIconSurface>,
}

impl ActiveDrag {
    pub(crate) fn is(&self, source: &WlDataSource) -> bool {
        self.source.inner() == source
    }

    pub(crate) fn bytes(&self, mime_type: &str) -> Option<&[u8]> {
        self.items
            .iter()
            .find(|(offered, _)| offered == mime_type)
            .map(|(_, bytes)| bytes.as_slice())
    }
}

impl WaylandPlatform {
    /// Starts dragging `source` out of `window` with the compositor drawing its icon; needs a
    /// held pointer button, otherwise the compositor cancels it at once.
    pub fn start_drag(&mut self, window: WindowId, source: DragSource) {
        let device = self
            .seats
            .iter()
            .find_map(|devices| devices.data_device.as_ref());
        let (Some(manager), Some(device), Some(origin)) = (
            self.standard.data_device.as_ref(),
            device,
            self.windows.get(&window),
        ) else {
            self.push_window_event(window, WindowEvent::DragEnded(DragOutcome::Cancelled));
            return;
        };
        let drag_source = manager.create_drag_and_drop_source(
            &self.queue_handle,
            source.items.iter().map(|(mime_type, _)| mime_type.as_str()),
            DndAction::Copy | DndAction::Move,
        );
        let icon = source
            .icon
            .as_ref()
            .and_then(|icon| DragIconSurface::new(self, icon));
        drag_source.start_drag(
            device,
            origin.wl_surface(),
            icon.as_ref().map(DragIconSurface::surface),
            self.input_serial,
        );
        if let Some(icon) = &icon {
            icon.present();
        }
        self.drag = Some(ActiveDrag {
            window,
            source: drag_source,
            items: source.items,
            _icon: icon,
        });
    }

    /// Ends the drag that `source` belongs to, if it is the current one.
    pub(crate) fn end_drag(&mut self, source: &WlDataSource, outcome: DragOutcome) {
        if let Some(drag) = self.drag.take_if(|drag| drag.is(source)) {
            self.push_window_event(drag.window, WindowEvent::DragEnded(outcome));
        }
    }
}
