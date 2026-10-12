use layouting::{Dimension, FlexDirection, Overflow};
use view::{Cx, Element, IntoProp, Kind, NodeId, Prop, SizeProperty, StyleProp, View};

use crate::util::spring;

/// The kind behind [`sidebar_collapse`].
pub struct SidebarCollapseKind<V> {
    inner: V,
    open: Prop<bool>,
}

fn height_when(open: bool) -> StyleProp {
    StyleProp::Size(
        SizeProperty::Height,
        if open {
            Dimension::Auto
        } else {
            Dimension::Length(0.0)
        },
    )
}

impl<V: View> Kind for SidebarCollapseKind<V> {
    const NAME: &'static str = "kit_sidebar_collapse";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        cx.set_default_style(node, StyleProp::FlexDirection(FlexDirection::Column));
        cx.set_default_style(node, StyleProp::Overflow(Overflow::Clip));
        match self.open {
            Prop::Static(open) => cx.set_default_style(node, height_when(open)),
            Prop::Dynamic(mut compute) => {
                cx.bind_style(node, move |runtime| height_when(compute(runtime)));
            }
        }
        let inner = cx.build(self.inner);
        cx.set_default_style(inner, StyleProp::Shrink(0.0));
        cx.adopt_children(node, vec![inner]);
    }
}

/// Wraps `inner` so its visible height springs between its natural size and nothing as `open`
/// changes; the child keeps its natural layout throughout and is only clipped.
pub fn sidebar_collapse<V: View>(
    inner: V,
    open: impl IntoProp<bool>,
) -> Element<SidebarCollapseKind<V>> {
    Element::new(SidebarCollapseKind {
        inner,
        open: open.into_prop(),
    })
    .transition(spring::smooth())
}
