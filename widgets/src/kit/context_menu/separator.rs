use view::{Cx, Element, Kind, NodeId, StyleProp, Styled};

use super::metrics::*;
use crate::{kit::use_theme, util::themed::bind_themed_style};

/// The kind behind [`menu_separator`].
pub struct MenuSeparatorKind;

impl Kind for MenuSeparatorKind {
    const NAME: &'static str = "kit_menu_separator";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        cx.set_role(node, semantics::Role::Separator);
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::Background(Some(theme.menu.separator))
        });
    }
}

/// A one-pixel line between sections of a menu, inset to the row labels.
pub fn menu_separator() -> Element<MenuSeparatorKind> {
    Element::new(MenuSeparatorKind)
        .self_stretch()
        .h(SEPARATOR_LINE)
        .mx(ROW_H_PADDING)
        .mt(SEPARATOR_SPACE_ABOVE)
        .mb(SEPARATOR_SPACE_BELOW)
        .shrink(0.0)
}
