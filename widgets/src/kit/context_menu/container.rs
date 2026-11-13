use semantics::Role;
use view::{Cx, Element, Kind, NodeId, Styled, ViewSeq};

use super::metrics::*;
use crate::{kit::use_theme, util::glass::glass_surface};

/// The kind behind [`context_menu`].
pub struct ContextMenuKind<Items> {
    items: Items,
}

impl<Items: ViewSeq> Kind for ContextMenuKind<Items> {
    const NAME: &'static str = "kit_context_menu";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        cx.set_role(node, Role::Menu);
        let plate = glass_surface(cx, node, theme, MENU_RADIUS);
        cx.set_children(node, (plate, self.items));
    }
}

/// Stacks menu rows in a frosted-glass panel that blurs what lies behind it and grows to fit
/// its widest row.
pub fn context_menu<Items: ViewSeq>(items: Items) -> Element<ContextMenuKind<Items>> {
    Element::new(ContextMenuKind { items })
        .flex_col()
        .items_stretch()
        .self_start()
        .min_w(MENU_MIN_WIDTH)
        .px(MENU_H_PADDING)
        .py(MENU_V_PADDING)
        .rounded(MENU_RADIUS)
        .border(BORDER_WIDTH)
        .shrink(0.0)
}
