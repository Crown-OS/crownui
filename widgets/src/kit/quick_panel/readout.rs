use view::{Cx, IntoProp, Kind, NodeId, Prop, StyleProp, Styled};

use super::{
    metrics::{
        BODY_TEXT, DISPLAY_TEXT, PAD_X, READOUT_HEIGHT, READOUT_PRIMARY_HEIGHT,
        READOUT_SECONDARY_HEIGHT,
    },
    typography::role_text,
};
use crate::{kit::use_theme, stack::hstack, util::themed::themed};

/// The kind behind [`panel_readout`](super::panel_readout).
pub struct PanelReadoutKind {
    pub(super) primary: Prop<String>,
    pub(super) secondary: Prop<String>,
}

impl Kind for PanelReadoutKind {
    const NAME: &'static str = "kit_panel_readout";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        for prop in [
            StyleProp::FlexDirection(layouting::FlexDirection::Column),
            StyleProp::Justify(layouting::Justify::Center),
            StyleProp::Padding(view::Edges::Horizontal, PAD_X),
            StyleProp::Size(
                view::SizeProperty::Height,
                layouting::Dimension::Length(READOUT_HEIGHT),
            ),
            StyleProp::Shrink(0.0),
        ] {
            cx.set_default_style(node, prop);
        }
        let line = |content: Prop<String>,
                    role,
                    height,
                    pick: fn(&crate::kit::Theme) -> geometry::Color| {
            hstack(role_text(content, role).text_color(themed(theme, pick)))
                .h(height)
                .items_center()
        };
        cx.set_children(
            node,
            (
                line(
                    self.primary,
                    DISPLAY_TEXT,
                    READOUT_PRIMARY_HEIGHT,
                    |theme| theme.popover.text,
                ),
                line(
                    self.secondary,
                    BODY_TEXT,
                    READOUT_SECONDARY_HEIGHT,
                    |theme| theme.popover.muted_text,
                ),
            ),
        );
    }
}

pub(super) fn readout_kind(
    primary: impl IntoProp<String>,
    secondary: impl IntoProp<String>,
) -> PanelReadoutKind {
    PanelReadoutKind {
        primary: primary.into_prop(),
        secondary: secondary.into_prop(),
    }
}
