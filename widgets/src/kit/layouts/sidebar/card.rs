use view::{Cx, Element, Kind, NodeId, StyleProp, Styled, ViewSeq};

use crate::{kit::use_theme, util::themed::bind_themed_style};

const CARD_RADIUS: f32 = 16.0;
const CARD_BORDER_WIDTH: f32 = 1.0;

/// The kind behind the sidebar's content card.
pub struct SidebarCardKind<Content> {
    content: Content,
}

impl<Content: ViewSeq> Kind for SidebarCardKind<Content> {
    const NAME: &'static str = "kit_sidebar_card";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        cx.set_default_style(
            node,
            StyleProp::FlexDirection(layouting::FlexDirection::Column),
        );
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::Background(Some(theme.surface.bg))
        });
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::BorderColor(theme.surface.border)
        });
        cx.set_children(node, self.content);
    }
}

pub(super) fn content_card<Content: ViewSeq>(
    content: Content,
) -> Element<SidebarCardKind<Content>> {
    Element::new(SidebarCardKind { content })
        .flex_1()
        .rounded(CARD_RADIUS)
        .border(CARD_BORDER_WIDTH)
}
