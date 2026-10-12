use reactive::{Signal, Store};
use view::Cx;

use super::entry::SelectCallback;
use crate::util::anchored_popup::AnchoredPopup;

/// The state every part of one popup menu shares: whether it is open, which row is highlighted
/// and what each row runs.
#[derive(Clone, Copy)]
pub(super) struct MenuHandles {
    pub(super) popup: AnchoredPopup,
    pub(super) highlighted: Signal<Option<usize>>,
    pub(super) callbacks: Store<Vec<Option<SelectCallback>>>,
}

impl MenuHandles {
    pub(super) fn close(self, cx: &mut Cx) {
        self.popup.dismiss(cx);
        self.highlighted.set(cx, None);
    }

    pub(super) fn pick(self, cx: &mut Cx, index: usize) {
        let taken = self
            .callbacks
            .update(cx, |all| all.get_mut(index).and_then(Option::take));
        if let Some(mut callback) = taken {
            callback(cx);
            self.callbacks.update(cx, |all| {
                if let Some(slot) = all.get_mut(index) {
                    *slot = Some(callback);
                }
            });
        }
        self.close(cx);
    }
}
