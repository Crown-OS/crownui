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

use super::{
    dnd_action::from_wayland_action,
    mime::{TEXT_MIME_TYPES, is_text_mime},
    pipe::write_in_background,
};
use crate::{DragOutcome, wayland::event_loop::WaylandPlatform};

/// What this client put on the clipboard, served until another client takes the selection.
pub(crate) struct OwnedSelection {
    source: CopyPasteSource,
    items: Vec<(String, Vec<u8>)>,
}

impl OwnedSelection {
    /// The bytes offered as `mime_type`; any text type is served from the item holding text.
    pub(crate) fn bytes(&self, mime_type: &str) -> Option<&[u8]> {
        let exact = self.items.iter().find(|(offered, _)| offered == mime_type);
        let text = || {
            is_text_mime(mime_type)
                .then(|| self.items.iter().find(|(offered, _)| is_text_mime(offered)))
                .flatten()
        };
        exact.or_else(text).map(|(_, bytes)| bytes.as_slice())
    }
}

impl WaylandPlatform {
    /// Offers `text` as the clipboard selection; needs the data-device global and a recent
    /// input serial, otherwise the compositor ignores it.
    pub fn set_clipboard_text(&mut self, text: String) {
        self.set_clipboard(vec![(TEXT_MIME_TYPES[0].to_owned(), text.into_bytes())]);
    }

    /// Offers `items`, the same content in several MIME types, as the clipboard selection; an
    /// item in a text type is offered under every text type.
    pub fn set_clipboard(&mut self, items: Vec<(String, Vec<u8>)>) {
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
        let has_text = items.iter().any(|(mime_type, _)| is_text_mime(mime_type));
        let text_aliases = TEXT_MIME_TYPES
            .into_iter()
            .filter(|alias| has_text && !items.iter().any(|(offered, _)| offered == alias));
        let offered = items
            .iter()
            .map(|(mime_type, _)| mime_type.as_str())
            .chain(text_aliases);
        let source = manager.create_copy_paste_source(&self.queue_handle, offered);
        source.set_selection(device, self.input_serial);
        self.clipboard = Some(OwnedSelection { source, items });
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
        if let Some(bytes) = self
            .clipboard
            .as_ref()
            .filter(|selection| selection.source.inner() == source)
            .and_then(|selection| selection.bytes(&mime_type))
        {
            write_in_background(pipe, bytes.to_vec());
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
        let action = self
            .drag
            .as_ref()
            .filter(|drag| drag.is(source))
            .map(|drag| drag.action)
            .unwrap_or_default();
        self.end_drag(source, DragOutcome::Dropped(action));
    }

    fn action(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        source: &WlDataSource,
        action: DndAction,
    ) {
        if let (Some(drag), Some(action)) = (
            self.drag.as_mut().filter(|drag| drag.is(source)),
            from_wayland_action(action),
        ) {
            drag.action = action;
        }
    }
}
