use geometry::{Point, Rect, Transform, Vector};
use graphics::{BoundaryId, CompositeOp, Composition, LayerProps};
use view::NodeKey;

use super::tree::RenderTree;

/// Walks the tree in paint order: each node is a layer placed by its frame, transform and opacity;
/// its own boundary draws first, then children inside its clip and scroll offset.
pub(crate) fn compose(tree: &RenderTree, clear_color: geometry::Color) -> Composition {
    let mut ops = Vec::with_capacity(tree.nodes.len() * 3);
    if let Some(root) = tree.root {
        compose_node(tree, root, &mut ops);
    }
    Composition { clear_color, ops }
}

fn compose_node(tree: &RenderTree, key: NodeKey, ops: &mut Vec<CompositeOp>) {
    let Some(node) = tree.nodes.get(&key) else {
        return;
    };
    if node.visual.hidden {
        return;
    }
    let values = node.values.current();
    if values.opacity <= 0.0 {
        return;
    }
    let frame = node.frame.current();
    let size = frame.size;
    let origin = Point::new(
        size.width * node.visual.origin.x,
        size.height * node.visual.origin.y,
    );
    let local = Transform::scale(values.scale.x, values.scale.y)
        .then(Transform::rotate(values.rotate.to_radians()))
        .about(origin);
    let placement = local.then(Transform::translate(
        frame.origin.to_vector() + values.translate,
    ));
    let own_shape = (Rect::new(Point::ZERO, size), values.radii);
    let backdrop = (values.backdrop_blur > 0.0).then_some(values.backdrop_blur);
    ops.push(CompositeOp::PushLayer(LayerProps {
        transform: placement,
        opacity: values.opacity.min(1.0),
        clip: backdrop.map(|_| own_shape),
        backdrop_blur: backdrop,
    }));
    if node.has_list {
        ops.push(CompositeOp::Draw(BoundaryId(key.0)));
    }
    if !node.children.is_empty() {
        let clip = node.visual.clip.then_some(own_shape);
        let scrolled = values.content_offset != Vector::ZERO;
        if clip.is_some() || scrolled {
            ops.push(CompositeOp::PushLayer(LayerProps {
                clip,
                ..LayerProps::default()
            }));
        }
        if scrolled {
            ops.push(CompositeOp::PushLayer(LayerProps {
                transform: Transform::translate(-values.content_offset),
                ..LayerProps::default()
            }));
        }
        let mut children: Vec<NodeKey> = node.children.clone();
        children.sort_by_key(|child| {
            tree.nodes
                .get(child)
                .map_or(0, |child| child.visual.z_index)
        });
        for child in children {
            compose_node(tree, child, ops);
        }
        if scrolled {
            ops.push(CompositeOp::PopLayer);
        }
        if clip.is_some() || scrolled {
            ops.push(CompositeOp::PopLayer);
        }
    }
    ops.push(CompositeOp::PopLayer);
}
