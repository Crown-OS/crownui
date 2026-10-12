use geometry::Size;
use graphics::DrawList;
use reactive::Runtime;
use view::{Cx, Element, Kind, NodeId};

/// The kind behind [`canvas`].
pub struct CanvasKind<P> {
    painter: P,
}

impl<P: FnMut(&mut Runtime, Size, &mut DrawList) + 'static> Kind for CanvasKind<P> {
    const NAME: &'static str = "canvas";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        cx.set_canvas(node, self.painter);
    }
}

/// Custom drawing: `painter` records into a [`DrawList`] in node-local coordinates and re-runs
/// whenever the node's size or any signal it reads changes. Give it a size with `.w()`/`.h()` or
/// let it grow.
pub fn canvas<P: FnMut(&mut Runtime, Size, &mut DrawList) + 'static>(
    painter: P,
) -> Element<CanvasKind<P>> {
    Element::new(CanvasKind { painter })
}
