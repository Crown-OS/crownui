use view::{Cx, Element, Kind, NodeId, StyleProp, Styled, ViewSeq};

use crate::{
    kit::use_theme,
    util::{spring, themed::bind_themed_style},
};

const SIDEBAR_WIDTH: f32 = 260.0;
const SIDEBAR_PADDING: f32 = 12.0;
const GROUP_GAP: f32 = 6.0;

/// The kind behind the sidebar's menu column.
pub struct SidebarMenuKind<Menus> {
    menus: Menus,
}

impl<Menus: ViewSeq> Kind for SidebarMenuKind<Menus> {
    const NAME: &'static str = "kit_sidebar_menu";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        for prop in [
            StyleProp::FlexDirection(layouting::FlexDirection::Column),
            StyleProp::Shrink(0.0),
        ] {
            cx.set_default_style(node, prop);
        }
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::Background(Some(theme.sidebar.bg))
        });
        cx.set_children(node, self.menus);
    }
}

pub(super) fn menu_column<Menus: ViewSeq>(menus: Menus) -> Element<SidebarMenuKind<Menus>> {
    Element::new(SidebarMenuKind { menus })
        .w(SIDEBAR_WIDTH)
        .p(SIDEBAR_PADDING)
        .gap(GROUP_GAP)
        .transition(spring::smooth())
}
