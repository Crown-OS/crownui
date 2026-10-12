use view::{Element, Styled, ViewSeq};

use super::{
    card::{SidebarCardKind, content_card},
    menu::{SidebarMenuKind, menu_column},
};
use crate::stack::{StackKind, hstack, vstack};

const LAYOUT_PADDING: f32 = 12.0;
const SIDEBAR_PADDING: f32 = 12.0;
const HEADER_GAP: f32 = 8.0;

type Shell<Menus, Body> = Element<StackKind<(Element<SidebarMenuKind<Menus>>, Body)>>;

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
    hstack((menu_column(menus), content_card(content)))
        .w_full()
        .h_full()
        .p(LAYOUT_PADDING)
}

/// [`sidebar`] with a toolbar row above the card, level with the menu's first row; place
/// `spacer()` in `header` to push what follows it to the trailing edge.
pub fn sidebar_with_header<Menus: ViewSeq, Header: ViewSeq, Content: ViewSeq>(
    menus: Menus,
    header: Header,
    content: Content,
) -> Shell<Menus, HeaderBody<Header, Content>> {
    let toolbar = hstack(header).gap(HEADER_GAP).pt(SIDEBAR_PADDING);
    let body = vstack((toolbar, content_card(content)))
        .gap(LAYOUT_PADDING)
        .flex_1();
    hstack((menu_column(menus), body))
        .w_full()
        .h_full()
        .p(LAYOUT_PADDING)
}
