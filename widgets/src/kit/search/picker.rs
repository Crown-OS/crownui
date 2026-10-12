use reactive::{Signal, Store};
use view::{Cx, NodeId};

use super::state::SearchState;
use crate::util::text_edit::Selection;

pub(super) type SelectHandler = Box<dyn FnMut(&mut Cx, usize)>;

#[derive(Clone, Copy)]
pub(super) struct Picker {
    pub(super) field: NodeId,
    pub(super) query: Signal<String>,
    pub(super) selection: Signal<Selection>,
    pub(super) state: SearchState,
    pub(super) on_select: Store<Option<SelectHandler>>,
}

impl Picker {
    pub(super) fn pick(self, cx: &mut Cx, index: usize) {
        let label = self.state.rows.with(cx, |rows| {
            rows.iter()
                .find(|(row, _)| *row == index)
                .map(|(_, suggestion)| suggestion.label.clone())
        });
        let Some(label) = label else {
            return;
        };
        self.state.dismiss(cx);
        match self.on_select.update(cx, Option::take) {
            Some(mut handler) => {
                handler(cx, index);
                self.on_select.update(cx, |slot| *slot = Some(handler));
            }
            None => {
                self.state.filled_by_pick.set(cx, true);
                self.selection.set(cx, Selection::at(label.len()));
                self.query.set(cx, label.clone());
                cx.emit_input(self.field, label);
            }
        }
    }
}
