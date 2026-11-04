use super::mime::{TEXT_MIME_TYPES, first_offered, preferred_text_mime};
use crate::{ClipboardData, ClipboardReadId, WindowEvent, wayland::event_loop::WaylandPlatform};

fn utf8_text(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes)
        .unwrap_or_else(|error| String::from_utf8_lossy(error.as_bytes()).into_owned())
}

impl WaylandPlatform {
    /// Reads the clipboard's text without blocking and delivers it as
    /// [`WindowEvent::ClipboardText`] to the keyboard-focused window.
    pub fn request_clipboard_text(&mut self) {
        let Some(window_id) = self.keyboard_focus else {
            return;
        };
        if let Some(selection) = &self.clipboard {
            let text = selection
                .bytes(TEXT_MIME_TYPES[0])
                .map(|bytes| utf8_text(bytes.to_vec()))
                .unwrap_or_default();
            self.push_window_event(window_id, WindowEvent::ClipboardText(text));
            return;
        }
        let pipe = self
            .seats
            .iter()
            .filter_map(|devices| devices.data_device.as_ref())
            .find_map(|device| device.data().selection_offer())
            .and_then(|offer| {
                let mime = offer.with_mime_types(preferred_text_mime)?;
                offer.receive(mime.to_owned()).ok()
            });
        let reading = pipe.is_some_and(|pipe| {
            self.read_pipe(pipe, move |state, bytes| {
                state.push_window_event(window_id, WindowEvent::ClipboardText(utf8_text(bytes)));
            })
        });
        if !reading {
            self.push_window_event(window_id, WindowEvent::ClipboardText(String::new()));
        }
    }
}

impl WaylandPlatform {
    /// Reads the clipboard in the first of `mime_types` it holds without blocking and delivers
    /// it as [`WindowEvent::ClipboardRead`] to the keyboard-focused window (or any window).
    pub fn read_clipboard(&mut self, request: ClipboardReadId, mime_types: Vec<String>) {
        let Some(window_id) = self
            .keyboard_focus
            .or_else(|| self.windows.keys().next().copied())
        else {
            return;
        };
        if let Some(selection) = &self.clipboard {
            let data = mime_types.into_iter().find_map(|mime_type| {
                let data = selection.bytes(&mime_type)?.to_vec();
                Some(ClipboardData { mime_type, data })
            });
            self.push_window_event(window_id, WindowEvent::ClipboardRead { request, data });
            return;
        }
        let pending = self
            .seats
            .iter()
            .filter_map(|devices| devices.data_device.as_ref())
            .find_map(|device| device.data().selection_offer())
            .and_then(|offer| {
                let mime_type = offer.with_mime_types(|offered| {
                    first_offered(&mime_types, offered).map(str::to_owned)
                })?;
                let pipe = offer.receive(mime_type.clone()).ok()?;
                Some((mime_type, pipe))
            });
        let reading = pending.is_some_and(|(mime_type, pipe)| {
            self.read_pipe(pipe, move |state, data| {
                let data = Some(ClipboardData { mime_type, data });
                state.push_window_event(window_id, WindowEvent::ClipboardRead { request, data });
            })
        });
        if !reading {
            let empty = WindowEvent::ClipboardRead {
                request,
                data: None,
            };
            self.push_window_event(window_id, empty);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_utf8_is_replaced_not_dropped() {
        assert_eq!(utf8_text(b"ok".to_vec()), "ok");
        assert_eq!(utf8_text(vec![b'a', 0xff]), "a\u{fffd}");
    }
}
