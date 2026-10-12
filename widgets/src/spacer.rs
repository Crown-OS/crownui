use view::{Cx, Element, Kind, NodeId, StyleProp};

/// The kind behind [`spacer`].
pub struct SpacerKind;

impl Kind for SpacerKind {
    const NAME: &'static str = "spacer";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        cx.set_default_style(node, StyleProp::Grow(1.0));
    }
}

/// Empty space that grows to fill its stack.
pub fn spacer() -> Element<SpacerKind> {
    Element::new(SpacerKind)
}
