use geometry::Color;
use layouting::Dimension;
use semantics::Role;
use view::{Cx, Element, Kind, NodeId, SizeProperty, StyleProp};

const DIVIDER_COLOR: Color = Color::new(0.5, 0.5, 0.5, 0.25);

/// The kind behind [`divider`].
pub struct DividerKind;

impl Kind for DividerKind {
    const NAME: &'static str = "divider";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        cx.set_role(node, Role::Separator);
        cx.set_default_style(
            node,
            StyleProp::Size(SizeProperty::Height, Dimension::Length(1.0)),
        );
        cx.set_default_style(node, StyleProp::Background(Some(DIVIDER_COLOR)));
        cx.set_default_style(node, StyleProp::Shrink(0.0));
    }
}

/// A one-pixel separator line.
pub fn divider() -> Element<DividerKind> {
    Element::new(DividerKind)
}
