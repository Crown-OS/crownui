use semantics::Role;
use view::{Cx, Element, Kind, Listener, NodeId, Semantic, Styled};

use super::{handles::SelectHandles, metrics::*};
use crate::text::text;

/// The kind behind one option row of a select's panel.
pub struct SelectRowKind {
    label: String,
    index: usize,
    handles: SelectHandles,
}

impl Kind for SelectRowKind {
    const NAME: &'static str = "kit_select_row";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let Self {
            label,
            index,
            handles,
        } = self;
        let selected = handles.selected;
        cx.set_role(node, Role::ListItem);
        cx.bind_checked(node, move |runtime| selected.get(runtime) == Some(index));
        cx.add_listener(
            node,
            Listener::PointerEnter(Box::new(move |cx, _| {
                handles.highlighted.set(cx, Some(index))
            })),
        );
        cx.add_listener(
            node,
            Listener::Click(Box::new(move |cx, event| {
                handles.pick(cx, index);
                event.stop_propagation();
            })),
        );
        cx.set_children(node, text(label).truncate().max_w(ROW_TEXT_WIDTH));
    }
}

pub(super) fn option_row(
    label: String,
    index: usize,
    handles: SelectHandles,
) -> Element<SelectRowKind> {
    Element::new(SelectRowKind {
        label: label.clone(),
        index,
        handles,
    })
    .label(label)
    .focusable(false)
    .flex_col()
    .justify_center()
    .h(ROW_HEIGHT)
    .px(ROW_H_PADDING)
    .shrink(0.0)
}
