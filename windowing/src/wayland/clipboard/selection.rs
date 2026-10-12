use sctk::{
    data_device_manager::{
        WritePipe,
        data_source::{CopyPasteSource, DataSourceHandler},
    },
    reexports::client::{
        Connection, QueueHandle,
        protocol::{wl_data_device_manager::DndAction, wl_data_source::WlDataSource},
    },
};

use super::{mime::TEXT_MIME_TYPES, pipe::write_in_background};
use crate::{DragOutcome, wayland::event_loop::WaylandPlatform};

/// Text this client put on the clipboard, served until another client takes the selection.
pub(crate) struct OwnedSelection {
    source: CopyPasteSource,
    pub(crate) text: String,
}

impl WaylandPlatform {
    /// Offers `text` as the clipboard selection; needs the data-device global and a recent
    /// input serial, otherwise the compositor ignores it.
    pub fn set_clipboard_text(&mut self, text: String) {
        let Some(manager) = &self.standard.data_device else {
            return;
        };
        let Some(device) = self
            .seats
            .iter()
            .find_map(|devices| devices.data_device.as_ref())
        else {
            return;
        };
        let source = manager.create_copy_paste_source(&self.queue_handle, TEXT_MIME_TYPES);
        source.set_selection(device, self.input_serial);
        self.clipboard = Some(OwnedSelection { source, text });
    }
}

impl DataSourceHandler for WaylandPlatform {
    fn accept_mime(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlDataSource,
        _: Option<String>,
    ) {
    }

    fn send_request(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        source: &WlDataSource,
        mime_type: String,
        pipe: WritePipe,
    ) {
        if let Some(selection) = self
            .clipboard
            .as_ref()
            .filter(|selection| selection.source.inner() == source)
        {
            write_in_background(pipe, selection.text.clone().into_bytes());
        } else if let Some(bytes) = self
            .drag
            .as_ref()
            .filter(|drag| drag.is(source))
            .and_then(|drag| drag.bytes(&mime_type))
        {
            write_in_background(pipe, bytes.to_vec());
        }
    }

    fn cancelled(&mut self, _: &Connection, _: &QueueHandle<Self>, source: &WlDataSource) {
        if self
            .clipboard
            .as_ref()
            .is_some_and(|selection| selection.source.inner() == source)
        {
            self.clipboard = None;
        }
        self.end_drag(source, DragOutcome::Cancelled);
    }

    fn dnd_dropped(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataSource) {}

    fn dnd_finished(&mut self, _: &Connection, _: &QueueHandle<Self>, source: &WlDataSource) {
        self.end_drag(source, DragOutcome::Dropped);
    }

    fn action(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataSource, _: DndAction) {}
}
