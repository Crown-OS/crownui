use super::mime::preferred_text_mime;
use crate::{WindowEvent, wayland::event_loop::WaylandPlatform};

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
            let text = selection.text.clone();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_utf8_is_replaced_not_dropped() {
        assert_eq!(utf8_text(b"ok".to_vec()), "ok");
        assert_eq!(utf8_text(vec![b'a', 0xff]), "a\u{fffd}");
    }
}
