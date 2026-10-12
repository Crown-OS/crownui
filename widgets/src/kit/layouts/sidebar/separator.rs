use semantics::Role;
use view::{Cx, Element, Kind, NodeId, StyleProp, Styled};

use crate::{kit::use_theme, stack::vstack, util::themed::themed};

const SEPARATOR_HEIGHT: f32 = 12.0;
const HAIRLINE_HEIGHT: f32 = 1.0;

/// The kind behind [`sidebar_separator`].
pub struct SidebarSeparatorKind;

impl Kind for SidebarSeparatorKind {
    const NAME: &'static str = "kit_sidebar_separator";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        cx.set_role(node, Role::Separator);
        cx.set_default_style(node, StyleProp::Shrink(0.0));
        let hairline = vstack(())
            .h(HAIRLINE_HEIGHT)
            .mt((SEPARATOR_HEIGHT - HAIRLINE_HEIGHT) / 2.0)
            .bg(themed(theme, |theme| theme.sidebar.separator));
        cx.set_children(node, hairline);
    }
}

/// A thin horizontal divider that chunks the sidebar into groups.
pub fn sidebar_separator() -> Element<SidebarSeparatorKind> {
    Element::new(SidebarSeparatorKind).h(SEPARATOR_HEIGHT)
}
