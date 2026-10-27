use geometry::{Point, Rect, Transform, Vector};
use graphics::{BoundaryId, CompositeOp, Composition, LayerProps};
use view::NodeKey;

use super::tree::RenderTree;

/// A top-layer subtree waiting to paint after the rest of the tree, outside its ancestors' clips.
struct TopLayer {
    key: NodeKey,
    parent_transform: Transform,
    parent_opacity: f32,
}

struct Composer<'a> {
    tree: &'a RenderTree,
    ops: Vec<CompositeOp>,
    top_layers: Vec<TopLayer>,
}

/// Walks the tree in paint order: each node is a layer placed by its frame, transform and opacity;
/// its own boundary draws first, then children inside its clip and scroll offset. Top-layer
/// subtrees paint last, in window space, so no ancestor clips them.
pub(crate) fn compose(tree: &RenderTree, clear_color: geometry::Color) -> Composition {
    let mut composer = Composer {
        tree,
        ops: Vec::with_capacity(tree.nodes.len() * 3),
        top_layers: Vec::new(),
    };
    if let Some(root) = tree.root {
        composer.compose_node(root, Transform::IDENTITY, 1.0);
    }
    let mut next = 0;
    while let Some(&TopLayer {
        key,
        parent_transform,
        parent_opacity,
    }) = composer.top_layers.get(next)
    {
        composer.ops.push(CompositeOp::PushLayer(LayerProps {
            transform: parent_transform,
            opacity: parent_opacity,
            ..LayerProps::default()
        }));
        composer.compose_node(key, parent_transform, parent_opacity);
        composer.ops.push(CompositeOp::PopLayer);
        next += 1;
    }
    Composition {
        clear_color,
        ops: composer.ops,
    }
}

impl Composer<'_> {
    fn compose_node(&mut self, key: NodeKey, parent_transform: Transform, parent_opacity: f32) {
        let Some(node) = self.tree.nodes.get(&key) else {
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
        let opacity = values.opacity.min(1.0);
        let own_shape = (Rect::new(Point::ZERO, size), values.radii);
        let backdrop = (values.backdrop_blur > 0.0).then_some(values.backdrop_blur);
        self.ops.push(CompositeOp::PushLayer(LayerProps {
            transform: placement,
            opacity,
            clip: backdrop.map(|_| own_shape),
            backdrop_blur: backdrop,
        }));
        if node.has_list {
            self.ops.push(CompositeOp::Draw(BoundaryId(key.0)));
        }
        if !node.children.is_empty() {
            let clip = node.visual.clip.then_some(own_shape);
            let scrolled = values.content_offset != Vector::ZERO;
            let scroll = Transform::translate(-values.content_offset);
            if clip.is_some() || scrolled {
                self.ops.push(CompositeOp::PushLayer(LayerProps {
                    clip,
                    ..LayerProps::default()
                }));
            }
            if scrolled {
                self.ops.push(CompositeOp::PushLayer(LayerProps {
                    transform: scroll,
                    ..LayerProps::default()
                }));
            }
            let children_transform = scroll.then(placement).then(parent_transform);
            let children_opacity = opacity * parent_opacity;
            let mut children: Vec<NodeKey> = node.children.clone();
            children.sort_by_key(|child| {
                self.tree
                    .nodes
                    .get(child)
                    .map_or(0, |child| child.visual.z_index)
            });
            for child in children {
                if self
                    .tree
                    .nodes
                    .get(&child)
                    .is_some_and(|child| child.visual.top_layer)
                {
                    self.top_layers.push(TopLayer {
                        key: child,
                        parent_transform: children_transform,
                        parent_opacity: children_opacity,
                    });
                } else {
                    self.compose_node(child, children_transform, children_opacity);
                }
            }
            if scrolled {
                self.ops.push(CompositeOp::PopLayer);
            }
            if clip.is_some() || scrolled {
                self.ops.push(CompositeOp::PopLayer);
            }
        }
        self.ops.push(CompositeOp::PopLayer);
    }
}
