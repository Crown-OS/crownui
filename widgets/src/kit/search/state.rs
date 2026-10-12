use reactive::{Runtime, Signal};
use view::Cx;

use super::suggestion::Suggestion;

pub(super) type IndexedSuggestion = (usize, Suggestion);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Step {
    Next,
    Previous,
}

#[derive(Clone, Copy)]
pub(super) struct SearchState {
    pub(super) focused: Signal<bool>,
    pub(super) dismissed: Signal<bool>,
    pub(super) filled_by_pick: Signal<bool>,
    pub(super) highlight: Signal<Option<usize>>,
    pub(super) rows: Signal<Vec<IndexedSuggestion>>,
}

impl SearchState {
    pub(super) fn new(cx: &mut Cx) -> Self {
        Self {
            focused: cx.signal(false),
            dismissed: cx.signal(false),
            filled_by_pick: cx.signal(false),
            highlight: cx.signal(None),
            rows: cx.signal(Vec::new()),
        }
    }

    pub(super) fn is_open(self, runtime: &mut Runtime) -> bool {
        self.focused.get(runtime)
            && !self.dismissed.get(runtime)
            && self.rows.with(runtime, |rows| !rows.is_empty())
    }

    pub(super) fn dismiss(self, cx: &mut Cx) {
        self.dismissed.set(cx, true);
        self.highlight.set(cx, None);
    }

    pub(super) fn resume(self, cx: &mut Cx) {
        self.dismissed.set(cx, false);
    }

    pub(super) fn edited(self, cx: &mut Cx) {
        if self.filled_by_pick.get(cx) {
            self.filled_by_pick.set(cx, false);
        } else {
            self.resume(cx);
        }
    }

    pub(super) fn focus_changed(self, cx: &mut Cx, focused: bool) {
        self.focused.set(cx, focused);
        self.resume(cx);
        self.highlight.set(cx, None);
    }

    pub(super) fn step(self, cx: &mut Cx, step: Step) {
        let count = self.rows.with(cx, Vec::len);
        if count == 0 || !self.focused.get(cx) {
            return;
        }
        self.resume(cx);
        let next = match (self.highlight.get(cx), step) {
            (None, Step::Next) => 0,
            (None, Step::Previous) => count - 1,
            (Some(row), Step::Next) => (row + 1) % count,
            (Some(row), Step::Previous) => (row + count - 1) % count,
        };
        self.highlight.set(cx, Some(next));
    }
}
