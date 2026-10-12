use semantics::Role;
use view::{Cx, Element, Kind, NodeId, Semantic, StyleProp, Styled};
use windowing::CursorIcon;

use super::{
    hover::track_hover,
    metrics::{
        CAPTION_TEXT, CHEVRON, CHEVRON_RIGHT, PAD_X, ROW_INSET, ROW_RADIUS, SECTION_DROP,
        SECTION_HEIGHT,
    },
    typography::role_text,
};
use crate::{
    kit::{icon::tinted_icon, use_theme},
    spacer::spacer,
    util::themed::themed,
};

/// The kind behind [`panel_section`](super::panel_section).
pub struct PanelSectionKind {
    pub(super) title: String,
    pub(super) disclosure: bool,
}

/// The options of a [`panel_section`](super::panel_section).
pub trait PanelSectionOptions: Sized {
    /// Makes the heading a link to more, with a chevron and a hover wash (macOS's "Other Networks").
    fn disclosure(self) -> Self;
}

impl PanelSectionOptions for Element<PanelSectionKind> {
    fn disclosure(self) -> Self {
        self.configure(|kind| kind.disclosure = true)
    }
}

impl Kind for PanelSectionKind {
    const NAME: &'static str = "kit_panel_section";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        for prop in [
            StyleProp::FlexDirection(layouting::FlexDirection::Row),
            StyleProp::AlignItems(layouting::Align::Center),
            StyleProp::Margin(view::Edges::Horizontal, ROW_INSET),
            StyleProp::Padding(view::Edges::Horizontal, PAD_X - ROW_INSET),
            StyleProp::Padding(view::Edges::Top, SECTION_DROP * 2.0),
            StyleProp::Size(
                view::SizeProperty::Height,
                layouting::Dimension::Length(SECTION_HEIGHT),
            ),
            StyleProp::Radius(view::Corners::All, ROW_RADIUS),
            StyleProp::Shrink(0.0),
        ] {
            cx.set_default_style(node, prop);
        }
        let title = role_text(self.title, CAPTION_TEXT)
            .role(Role::Heading)
            .text_color(themed(theme, |theme| theme.popover.muted_text));
        if !self.disclosure {
            cx.set_children(node, title);
            return;
        }
        cx.set_default_style(node, StyleProp::Cursor(CursorIcon::Pointer));
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
        let chevron = tinted_icon(
            CHEVRON_RIGHT,
            2.0,
            themed(theme, |theme| theme.popover.muted_text),
        )
        .size(CHEVRON);
        cx.set_children(node, (title, spacer(), chevron));
    }
}
