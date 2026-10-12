use view::{Cx, Kind, NodeId, StyleProp, Styled};

use super::metrics::{PAD_X, SEPARATOR_HEIGHT};
use crate::{kit::use_theme, stack::hstack, util::themed::themed};

/// The kind behind [`panel_separator`](super::panel_separator).
pub struct PanelSeparatorKind;

impl Kind for PanelSeparatorKind {
    const NAME: &'static str = "kit_panel_separator";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        for prop in [
            StyleProp::FlexDirection(layouting::FlexDirection::Row),
            StyleProp::AlignItems(layouting::Align::Start),
            StyleProp::Padding(view::Edges::Horizontal, PAD_X),
            StyleProp::Padding(view::Edges::Top, SEPARATOR_HEIGHT / 2.0),
            StyleProp::Size(
                view::SizeProperty::Height,
                layouting::Dimension::Length(SEPARATOR_HEIGHT),
            ),
            StyleProp::Shrink(0.0),
        ] {
            cx.set_default_style(node, prop);
        }
        cx.set_children(
            node,
            hstack(())
                .h(1.0)
                .grow(1.0)
                .bg(themed(theme, |theme| theme.menu.separator)),
        );
    }
}
