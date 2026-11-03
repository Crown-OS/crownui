use reactive::Signal;
use semantics::Role;
use view::{AnyView, Cx, Element, Kind, NodeId, Semantic, StyleProp, Styled, View};

use super::{
    metrics::{HEADER_HEIGHT, PAD_X, TITLE_TEXT, TRAILING_GAP},
    typography::role_text,
};
use crate::{
    kit::{toggle, use_theme},
    spacer::spacer,
    stack::hstack,
    util::themed::themed,
};

/// The kind behind [`panel_header`](super::panel_header).
pub struct PanelHeaderKind {
    pub(super) title: String,
    pub(super) switch: Option<Signal<bool>>,
    pub(super) trailing: Vec<AnyView>,
}

/// The options of a [`panel_header`](super::panel_header).
pub trait PanelHeaderOptions: Sized {
    /// Puts the panel's master switch, bound to `on`, at the header's right edge.
    fn switch(self, on: Signal<bool>) -> Self;
    /// Views between the title and the switch, e.g. a refresh button.
    fn trailing(self, view: impl View + 'static) -> Self;
}

impl PanelHeaderOptions for Element<PanelHeaderKind> {
    fn switch(self, on: Signal<bool>) -> Self {
        self.configure(|kind| kind.switch = Some(on))
    }

    fn trailing(self, view: impl View + 'static) -> Self {
        self.configure(|kind| kind.trailing.push(view.boxed()))
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
        let mut children: Vec<AnyView> = Vec::with_capacity(3 + self.trailing.len());
        children.push(title.boxed());
        children.push(spacer().boxed());
        children.extend(
            self.trailing
                .into_iter()
                .map(|view| hstack(view).shrink(0.0).ml(TRAILING_GAP).boxed()),
        );
        if let Some(on) = self.switch {
            children.push(
                toggle(on)
                    .label(self.title)
                    .self_center()
                    .ml(TRAILING_GAP)
                    .boxed(),
            );
        }
        cx.set_children(node, children);
    }
}
