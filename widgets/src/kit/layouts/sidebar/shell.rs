use view::{Cx, Element, Kind, NodeId, StyleProp, Styled, ViewSeq};

use super::{
    card::{SidebarCardKind, content_card},
    menu::{SidebarMenuKind, menu_column},
};
use crate::{
    kit::use_theme,
    stack::{StackKind, hstack, vstack},
    util::{spring, themed::bind_themed_style},
};

pub(super) const LAYOUT_PADDING: f32 = 6.0;
const HEADER_GAP: f32 = 8.0;

type Shell<Menus, Body> = Element<SidebarShellKind<(Element<SidebarMenuKind<Menus>>, Body)>>;

/// The kind behind [`sidebar`] and [`sidebar_with_header`]: a row painted in the title bar's
/// color, so the menu column and the toolbar read as the window's frame around the card.
pub struct SidebarShellKind<Children> {
    children: Children,
}

impl<Children: ViewSeq> Kind for SidebarShellKind<Children> {
    const NAME: &'static str = "kit_sidebar_shell";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        cx.set_default_style(
            node,
            StyleProp::FlexDirection(layouting::FlexDirection::Row),
        );
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::Background(Some(theme.sidebar.bg))
        });
        cx.set_children(node, self.children);
    }
}

fn shell<Menus: ViewSeq, Body: ViewSeq>(menus: Menus, body: Body) -> Shell<Menus, Body> {
    Element::new(SidebarShellKind {
        children: (menu_column(menus), body),
    })
    .w_full()
    .h_full()
    .p(LAYOUT_PADDING)
    .transition(spring::smooth())
}

type HeaderBody<Header, Content> = Element<
    StackKind<(
        Element<StackKind<Header>>,
        Element<SidebarCardKind<Content>>,
    )>,
>;

/// A menu column on the left and a rounded content card filling the rest.
pub fn sidebar<Menus: ViewSeq, Content: ViewSeq>(
    menus: Menus,
    content: Content,
) -> Shell<Menus, Element<SidebarCardKind<Content>>> {
    shell(menus, content_card(content))
}

/// [`sidebar`] with a toolbar row above the card, spaced from the window's top edge and from the
/// card by the layout's padding; for a window
/// with client-side decorations it is the title bar: lead `header` with a `title_bar_spacer()`
/// to drag the window by and end it with `window_controls()`.
pub fn sidebar_with_header<Menus: ViewSeq, Header: ViewSeq, Content: ViewSeq>(
    menus: Menus,
    header: Header,
    content: Content,
) -> Shell<Menus, HeaderBody<Header, Content>> {
    let toolbar = hstack(header).gap(HEADER_GAP);
    let body = vstack((toolbar, content_card(content)))
        .gap(LAYOUT_PADDING)
        .flex_1();
    shell(menus, body)
}
