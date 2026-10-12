use reactive::Signal;
use semantics::Role;
use view::{Cx, Element, Kind, NodeId, Semantic, StyleProp, Styled};

use super::{
    metrics::{HEADER_HEIGHT, PAD_X, TITLE_TEXT},
    typography::role_text,
};
use crate::{
    kit::{toggle, use_theme},
    spacer::spacer,
    util::themed::themed,
};

/// The kind behind [`panel_header`](super::panel_header).
pub struct PanelHeaderKind {
    pub(super) title: String,
    pub(super) switch: Option<Signal<bool>>,
}

/// The options of a [`panel_header`](super::panel_header).
pub trait PanelHeaderOptions: Sized {
    /// Puts the panel's master switch, bound to `on`, at the header's right edge.
    fn switch(self, on: Signal<bool>) -> Self;
}

impl PanelHeaderOptions for Element<PanelHeaderKind> {
    fn switch(self, on: Signal<bool>) -> Self {
        self.configure(|kind| kind.switch = Some(on))
    }
}

impl Kind for PanelHeaderKind {
    const NAME: &'static str = "kit_panel_header";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        for prop in [
            StyleProp::FlexDirection(layouting::FlexDirection::Row),
            StyleProp::AlignItems(layouting::Align::Center),
            StyleProp::Padding(view::Edges::Horizontal, PAD_X),
            StyleProp::Size(
                view::SizeProperty::Height,
                layouting::Dimension::Length(HEADER_HEIGHT),
            ),
            StyleProp::Shrink(0.0),
        ] {
            cx.set_default_style(node, prop);
        }
        let title = role_text(self.title.clone(), TITLE_TEXT)
            .role(Role::Heading)
            .text_color(themed(theme, |theme| theme.popover.text));
        match self.switch {
            Some(on) => cx.set_children(
                node,
                (title, spacer(), toggle(on).label(self.title).self_center()),
            ),
            None => cx.set_children(node, title),
        }
    }
}
