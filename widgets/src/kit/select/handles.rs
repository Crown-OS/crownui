use reactive::{Runtime, Signal};
use view::{Cx, NodeId};

use crate::util::anchored_popup::AnchoredPopup;

/// The state every part of one select shares.
#[derive(Clone, Copy)]
pub(super) struct SelectHandles {
    pub(super) popup: AnchoredPopup,
    pub(super) options: Signal<Vec<String>>,
    pub(super) placeholder: Signal<String>,
    pub(super) selected: Signal<Option<usize>>,
    pub(super) highlighted: Signal<Option<usize>>,
    pub(super) typed: Signal<String>,
    pub(super) root: NodeId,
}

/// What opened the panel: a pointer leaves it unhighlighted, anything else starts on the
/// selected row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OpenedBy {
    Pointer,
    Keyboard,
}

impl SelectHandles {
    pub(super) fn close(self, runtime: &mut Runtime) {
        self.popup.dismiss(runtime);
        self.highlighted.set(runtime, None);
        self.typed.update(runtime, String::clear);
    }

    pub(super) fn option_count(self, runtime: &mut Runtime) -> usize {
        self.options.with(runtime, Vec::len)
    }

    pub(super) fn open(self, cx: &mut Cx, highlight: Option<usize>) {
        let count = self.option_count(cx);
        if count == 0 {
            return;
        }
        self.popup.reveal(cx, super::metrics::panel_height(count));
        self.highlighted.set(cx, highlight);
    }

    pub(super) fn toggle(self, cx: &mut Cx, opened_by: OpenedBy) {
        if self.popup.is_open(cx) {
            self.close(cx);
        } else {
            let highlight = match opened_by {
                OpenedBy::Pointer => None,
                OpenedBy::Keyboard => self.selected.get(cx),
            };
            self.open(cx, highlight);
        }
    }

    pub(super) fn choose(self, cx: &mut Cx, index: usize) {
        let label = self.options.with(cx, |options| options.get(index).cloned());
        let Some(label) = label else {
            return;
        };
        self.selected.set(cx, Some(index));
        cx.emit_change(self.root, label);
    }

    pub(super) fn pick(self, cx: &mut Cx, index: usize) {
        self.choose(cx, index);
        self.close(cx);
    }
}
