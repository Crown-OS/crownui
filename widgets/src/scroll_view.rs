use layouting::Overflow;
use semantics::Role;
use view::{Cx, Element, Kind, NodeId, StyleProp, View};

/// The kind behind [`scroll_view`].
pub struct ScrollViewKind<V> {
    content: V,
}

impl<V: View> Kind for ScrollViewKind<V> {
    const NAME: &'static str = "scroll_view";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        cx.set_role(node, Role::ScrollView);
        cx.set_default_style(node, StyleProp::Overflow(Overflow::Scroll));
        let content = cx.build(self.content);
        cx.set_default_style(content, StyleProp::Shrink(0.0));
        cx.adopt_children(node, vec![content]);
    }
}

/// Clips `content` and scrolls it with the wheel or touchpad. Constrain its size (e.g. `.h(300.0)`
/// or `.grow(1.0)`) so the content can overflow.
pub fn scroll_view<V: View>(content: V) -> Element<ScrollViewKind<V>> {
    Element::new(ScrollViewKind { content })
}
