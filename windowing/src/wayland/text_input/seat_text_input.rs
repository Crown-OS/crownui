use sctk::reexports::{
    client::{QueueHandle, protocol::wl_seat::WlSeat},
    protocols::wp::text_input::zv3::client::{
        zwp_text_input_manager_v3::ZwpTextInputManagerV3,
        zwp_text_input_v3::{ChangeCause, ZwpTextInputV3},
    },
};

use super::{TextInputData, content_type::content_type, transaction::TextInputTransaction};
use crate::{
    TextInputState, WindowId,
    wayland::{event_loop::WaylandPlatform, util::surrounding_text::surrounding_window},
};

/// text-input-v3 caps surrounding text below the 4 KiB Wayland message size.
const MAX_SURROUNDING_BYTES: usize = 4000;

/// One seat's text-input object: the surface it is focused on, whether it is enabled there,
/// and the composition waiting for `done`.
pub(crate) struct SeatTextInput {
    proxy: ZwpTextInputV3,
    pub(crate) focus: Option<WindowId>,
    enabled: bool,
    pub(crate) transaction: TextInputTransaction,
}

impl SeatTextInput {
    pub(crate) fn new(
        manager: &ZwpTextInputManagerV3,
        seat: &WlSeat,
        qh: &QueueHandle<WaylandPlatform>,
    ) -> Self {
        Self {
            proxy: manager.get_text_input(seat, qh, TextInputData),
            focus: None,
            enabled: false,
            transaction: TextInputTransaction::default(),
        }
    }

    pub(crate) fn owns(&self, proxy: &ZwpTextInputV3) -> bool {
        &self.proxy == proxy
    }

    /// Enables the input method with `state`, refreshes it, or disables it for `None`.
    pub(crate) fn send(&mut self, state: Option<&TextInputState>) {
        let Some(state) = state else {
            if std::mem::take(&mut self.enabled) {
                self.proxy.disable();
                self.commit();
            }
            return;
        };
        if !std::mem::replace(&mut self.enabled, true) {
            self.proxy.enable();
        }
        if let Some((text, cursor, anchor)) = &state.surrounding_text {
            let (window, cursor, anchor) =
                surrounding_window(text, *cursor, *anchor, MAX_SURROUNDING_BYTES);
            self.proxy.set_surrounding_text(
                window.to_owned(),
                i32::try_from(cursor).unwrap_or(i32::MAX),
                i32::try_from(anchor).unwrap_or(i32::MAX),
            );
            self.proxy.set_text_change_cause(ChangeCause::Other);
        }
        let (hint, purpose) = content_type(state.content_purpose);
        self.proxy.set_content_type(hint, purpose);
        let area = state.cursor_area;
        self.proxy.set_cursor_rectangle(
            area.min_x().round() as i32,
            area.min_y().round() as i32,
            area.size.width.round() as i32,
            area.size.height.round() as i32,
        );
        self.commit();
    }

    /// Focus left: the compositor ignores requests until the next enter, so nothing is sent.
    pub(crate) fn leave(&mut self) {
        self.focus = None;
        self.enabled = false;
    }

    fn commit(&self) {
        self.proxy.commit();
    }
}

impl Drop for SeatTextInput {
    fn drop(&mut self) {
        self.proxy.destroy();
    }
}
