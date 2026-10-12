use semantics::Role;
use view::{Cx, Kind, NodeId, StyleProp, Styled};
use windowing::CursorIcon;

use super::{
    hover::track_hover,
    metrics::{ACTION_HEIGHT, BODY_TEXT, PAD_X, ROW_INSET, ROW_RADIUS},
    typography::role_text,
};
use crate::{kit::use_theme, util::themed::themed};

/// The kind behind [`panel_action`](super::panel_action).
pub struct PanelActionKind {
    pub(super) label: String,
}

impl Kind for PanelActionKind {
    const NAME: &'static str = "kit_panel_action";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        cx.set_role(node, Role::Button);
        for prop in [
            StyleProp::FlexDirection(layouting::FlexDirection::Row),
            StyleProp::AlignItems(layouting::Align::Center),
            StyleProp::Margin(view::Edges::Horizontal, ROW_INSET),
            StyleProp::Padding(view::Edges::Horizontal, PAD_X - ROW_INSET),
            StyleProp::Size(
                view::SizeProperty::Height,
                layouting::Dimension::Length(ACTION_HEIGHT),
            ),
            StyleProp::Radius(view::Corners::All, ROW_RADIUS),
            StyleProp::Cursor(CursorIcon::Pointer),
            StyleProp::Shrink(0.0),
        ] {
            cx.set_default_style(node, prop);
        }
        let hovered = track_hover(cx, node);
        cx.bind_style(node, move |runtime| {
            let hovered = hovered.get(runtime);
            StyleProp::Background(Some(theme.with(runtime, |theme| {
                if hovered {
                    theme.popover.hover_bg
                } else {
                    geometry::TRANSPARENT
                }
            })))
        });
        cx.set_children(
            node,
            role_text(self.label, BODY_TEXT).text_color(themed(theme, |theme| theme.popover.text)),
        );
    }
}
