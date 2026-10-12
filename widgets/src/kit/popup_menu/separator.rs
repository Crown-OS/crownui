use semantics::Role;
use view::{Cx, Element, Kind, NodeId, StyleProp, Styled};

use super::metrics::*;
use crate::{kit::use_theme, util::themed::bind_themed_style};

/// The kind behind a popup menu's separator row.
pub struct PopupMenuSeparatorKind;

impl Kind for PopupMenuSeparatorKind {
    const NAME: &'static str = "kit_popup_menu_separator";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        cx.set_role(node, Role::Separator);
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::Background(Some(theme.menu.separator))
        });
    }
}

pub(super) fn separator_row() -> Element<PopupMenuSeparatorKind> {
    Element::new(PopupMenuSeparatorKind)
        .self_stretch()
        .h(SEPARATOR_LINE)
        .my((SEPARATOR_HEIGHT - SEPARATOR_LINE) / 2.0)
        .shrink(0.0)
}
