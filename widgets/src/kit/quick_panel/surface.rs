use view::{Cx, Kind, NodeId, StyleProp, ViewSeq};

use super::{
    metrics::{PAD_Y, PANEL_OPACITY, PANEL_RADIUS, PANEL_WIDTH},
    palette::{default_fill, panel_rim},
};
use crate::kit::{INTER_FAMILY, use_theme};

/// The kind behind [`quick_panel`](super::quick_panel).
pub struct QuickPanelKind<C> {
    pub(super) rows: C,
}

impl<C: ViewSeq> Kind for QuickPanelKind<C> {
    const NAME: &'static str = "kit_quick_panel";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        for prop in [
            StyleProp::FlexDirection(layouting::FlexDirection::Column),
            StyleProp::Size(
                view::SizeProperty::Width,
                layouting::Dimension::Length(PANEL_WIDTH),
            ),
            StyleProp::Padding(view::Edges::Vertical, PAD_Y),
            StyleProp::Radius(view::Corners::All, PANEL_RADIUS),
            StyleProp::FontFamily(INTER_FAMILY.to_owned()),
        ] {
            cx.set_default_style(node, prop);
        }
        cx.bind_style(node, move |runtime| {
            StyleProp::Background(Some(theme.with(runtime, default_fill)))
        });
        cx.bind_style(node, move |runtime| {
            StyleProp::Ring(Some((
                1.0,
                theme.with(runtime, |theme| panel_rim(theme, PANEL_OPACITY)),
            )))
        });
        cx.set_children(node, self.rows);
    }
}
