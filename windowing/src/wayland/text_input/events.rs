use sctk::{
    dispatch2::Dispatch2,
    reexports::{
        client::{Connection, QueueHandle},
        protocols::wp::text_input::zv3::client::zwp_text_input_v3::{self, ZwpTextInputV3},
    },
};

use crate::{
    WindowEvent,
    wayland::{
        event_loop::{QueuedEvent, WaylandPlatform},
        window::surface_window_id,
    },
};

impl WaylandPlatform {
    /// Sends each window's changed input-method request to the seats focused on it.
    pub(crate) fn apply_text_input_changes(&mut self) {
        for devices in &mut self.seats {
            let Some(text_input) = devices.text_input.as_mut() else {
                continue;
            };
            let Some(window) = text_input.focus.and_then(|id| self.windows.get_mut(&id)) else {
                continue;
            };
            if window.take_text_input_change() {
                text_input.send(window.text_input());
            }
        }
    }
}

/// User data of every `zwp_text_input_v3`; the owning seat is found by comparing proxies.
pub(crate) struct TextInputData;

impl Dispatch2<ZwpTextInputV3, WaylandPlatform> for TextInputData {
    fn event(
        &self,
        state: &mut WaylandPlatform,
        proxy: &ZwpTextInputV3,
        event: zwp_text_input_v3::Event,
        _: &Connection,
        _: &QueueHandle<WaylandPlatform>,
    ) {
        let WaylandPlatform {
            seats,
            windows,
            pending_events,
            ..
        } = state;
        let Some(text_input) = seats
            .iter_mut()
            .filter_map(|devices| devices.text_input.as_mut())
            .find(|text_input| text_input.owns(proxy))
        else {
            return;
        };
        match event {
            zwp_text_input_v3::Event::Enter { surface } => {
                let window_id = surface_window_id(&surface);
                let Some(window) = windows.get_mut(&window_id) else {
                    return;
                };
                text_input.focus = Some(window_id);
                window.take_text_input_change();
                text_input.send(window.text_input());
            }
            zwp_text_input_v3::Event::Leave { .. } => {
                if let (Some(window_id), Some(event)) =
                    (text_input.focus, text_input.transaction.reset())
                {
                    pending_events.push(QueuedEvent::Window(
                        window_id,
                        WindowEvent::TextInput(event),
                    ));
                }
                text_input.leave();
            }
            zwp_text_input_v3::Event::PreeditString {
                text,
                cursor_begin,
                cursor_end,
            } => text_input
                .transaction
                .set_preedit(text, cursor_begin, cursor_end),
            zwp_text_input_v3::Event::CommitString { text } => {
                text_input.transaction.set_commit(text);
            }
            zwp_text_input_v3::Event::DeleteSurroundingText {
                before_length,
                after_length,
            } => text_input
                .transaction
                .set_delete(before_length, after_length),
            zwp_text_input_v3::Event::Done { .. } => {
                let Some(window_id) = text_input.focus else {
                    text_input.transaction.reset();
                    return;
                };
                text_input.transaction.done(|event| {
                    pending_events.push(QueuedEvent::Window(
                        window_id,
                        WindowEvent::TextInput(event),
                    ));
                });
            }
            _ => {}
        }
    }
}
