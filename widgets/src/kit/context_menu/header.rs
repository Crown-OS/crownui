use view::{Cx, Element, Kind, NodeId, Prop, StyleProp, Styled};

use super::metrics::*;
use crate::{
    kit::{INTER_FAMILY, use_theme},
    util::themed::bind_themed_style,
};

/// The kind behind [`menu_header`].
pub struct MenuHeaderKind {
    label: String,
}

impl Kind for MenuHeaderKind {
    const NAME: &'static str = "kit_menu_header";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        cx.set_text(node, Prop::Static(self.label));
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::TextColor(theme.menu.header_text)
        });
    }
}

/// A section header: small medium-weight text aligned with the row labels.
pub fn menu_header(label: impl Into<String>) -> Element<MenuHeaderKind> {
    Element::new(MenuHeaderKind {
        label: label.into(),
    })
    .self_stretch()
    .h(HEADER_HEIGHT)
    .pt(HEADER_V_PADDING)
    .pb(HEADER_V_PADDING)
    .pl(ROW_H_PADDING + ICON_SLOT_WIDTH)
    .pr(ROW_H_PADDING)
    .shrink(0.0)
    .text_size(HEADER_FONT_SIZE)
    .font_medium()
    .font_family(INTER_FAMILY)
}
