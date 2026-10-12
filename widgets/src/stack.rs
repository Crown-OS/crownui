use layouting::{FlexDirection, Position};
use view::{Cx, Element, Kind, NodeId, StyleProp, ViewSeq};

/// Direction a stack lays its children out in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackAxis {
    /// Top to bottom.
    Vertical,
    /// Start to end.
    Horizontal,
    /// On top of each other.
    Depth,
}

/// The kind behind [`vstack`], [`hstack`] and [`zstack`].
pub struct StackKind<C> {
    axis: StackAxis,
    children: C,
}

impl<C: ViewSeq> Kind for StackKind<C> {
    const NAME: &'static str = "stack";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let direction = match self.axis {
            StackAxis::Horizontal => FlexDirection::Row,
            StackAxis::Vertical | StackAxis::Depth => FlexDirection::Column,
        };
        cx.set_default_style(node, StyleProp::FlexDirection(direction));
        let mut built = Vec::new();
        self.children.build_into(cx, &mut built);
        if self.axis == StackAxis::Depth {
            // An overlay its author already positioned keeps its own insets.
            let unplaced = |cx: &Cx, overlay: NodeId| {
                cx.tree()
                    .get(overlay)
                    .is_some_and(|element| element.style().layout.position != Position::Absolute)
            };
            for overlay in built.iter().skip(1).copied() {
                if unplaced(cx, overlay) {
                    cx.set_default_style(overlay, StyleProp::Position(Position::Absolute));
                    cx.set_default_style(overlay, StyleProp::Inset(view::Edges::All, 0.0));
                }
            }
        }
        cx.adopt_children(node, built);
    }
}

/// Children top to bottom (`flex flex-col`).
pub fn vstack<C: ViewSeq>(children: C) -> Element<StackKind<C>> {
    Element::new(StackKind {
        axis: StackAxis::Vertical,
        children,
    })
}

/// Children start to end (`flex flex-row`).
pub fn hstack<C: ViewSeq>(children: C) -> Element<StackKind<C>> {
    Element::new(StackKind {
        axis: StackAxis::Horizontal,
        children,
    })
}

/// Children layered on top of each other; the first one sizes the stack.
pub fn zstack<C: ViewSeq>(children: C) -> Element<StackKind<C>> {
    Element::new(StackKind {
        axis: StackAxis::Depth,
        children,
    })
}
