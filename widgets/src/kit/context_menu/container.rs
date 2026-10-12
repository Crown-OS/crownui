use semantics::Role;
use view::{Cx, Element, Kind, NodeId, StyleProp, Styled, ViewSeq};

use super::metrics::*;
use crate::{kit::use_theme, util::themed::bind_themed_style};

/// The kind behind [`context_menu`].
pub struct ContextMenuKind<Items> {
    items: Items,
}

impl<Items: ViewSeq> Kind for ContextMenuKind<Items> {
    const NAME: &'static str = "kit_context_menu";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        cx.set_role(node, Role::Menu);
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::Background(Some(theme.menu.bg))
        });
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::BorderColor(theme.menu.border)
        });
        cx.set_children(node, self.items);
    }
}

/// Stacks menu rows in the 340px rounded panel of the reference design.
pub fn context_menu<Items: ViewSeq>(items: Items) -> Element<ContextMenuKind<Items>> {
    Element::new(ContextMenuKind { items })
        .flex_col()
        .items_stretch()
        .self_start()
        .w(MENU_WIDTH)
        .px(BORDER_WIDTH)
        .py(MENU_V_PADDING + BORDER_WIDTH)
        .gap(MENU_GAP)
        .rounded(MENU_RADIUS)
        .border(BORDER_WIDTH)
        .shrink(0.0)
}
