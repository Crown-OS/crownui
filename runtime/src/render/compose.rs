use geometry::{CornerRadii, Point, Rect, Transform, Vector};
use graphics::{BoundaryId, CompositeOp, Composition, LayerProps};
use view::NodeKey;

use super::{
    Painted,
    tree::{RenderNode, RenderTree},
};

/// A top-layer subtree waiting to paint after the rest of the tree, outside its ancestors' clips.
struct TopLayer {
    key: NodeKey,
    parent_transform: Transform,
    parent_opacity: f32,
}

struct Composer<'a> {
    tree: &'a mut RenderTree,
    ops: Vec<CompositeOp>,
    top_layers: Vec<TopLayer>,
}

/// Walks the tree in paint order: each node is a layer placed by its frame, transform and opacity;
/// its own boundary draws first, then children inside its clip and scroll offset. Top-layer
/// subtrees paint last, in window space, so no ancestor clips them.
pub(crate) fn compose(tree: &mut RenderTree, clear_color: geometry::Color) -> Composition {
    let capacity = tree.nodes.len() * 3;
    tree.tick += 1;
    let mut composer = Composer {
        tree,
        ops: Vec::with_capacity(capacity),
        top_layers: Vec::new(),
    };
    if let Some(root) = composer.tree.root {
        composer.compose_node(root, Transform::IDENTITY, 1.0, None);
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
        composer.compose_node(key, parent_transform, parent_opacity, None);
        composer.ops.push(CompositeOp::PopLayer);
        next += 1;
    }
    Composition {
        clear_color,
        ops: composer.ops,
    }
}

/// Where `node` sits in its parent: its frame, transform and transform origin.
fn placement(node: &RenderNode) -> Transform {
    let values = node.values.current();
    let frame = node.frame.current();
    let offset = Transform::translate(frame.origin.to_vector() + values.translate);
    if values.scale == Vector::new(1.0, 1.0) && values.rotate == 0.0 {
        return offset;
    }
    let origin = Point::new(
        frame.size.width * node.visual.origin.x,
        frame.size.height * node.visual.origin.y,
    );
    Transform::scale(values.scale.x, values.scale.y)
        .then(Transform::rotate(values.rotate.to_radians()))
        .about(origin)
        .then(offset)
}

impl Composer<'_> {
    /// Where `child` draws in its parent's coordinates when, unchanged since it was last composed,
    /// it draws nothing inside `visible`, the part of its parent's content an ancestor's clip lets
    /// through: a row scrolled out of view.
    fn outside(&self, child: NodeKey, visible: Rect) -> Option<Painted> {
        let node = self.tree.nodes.get(&child)?;
        let painted = node.subtree?.transformed(placement(node));
        (painted.clipped(visible) == Painted::Nothing).then_some(painted)
    }

    /// Composes `key`'s subtree, of which ancestors' clips let through `visible` (in its parent's
    /// content coordinates, `None` when unbounded); returns where it draws in its parent's
    /// coordinates.
    fn compose_node(
        &mut self,
        key: NodeKey,
        parent_transform: Transform,
        parent_opacity: f32,
        visible: Option<Rect>,
    ) -> Painted {
        let top_layers = self.top_layers.len();
        let painted = self.compose_subtree(key, parent_transform, parent_opacity, visible);
        let Some(node) = self.tree.nodes.get_mut(&key) else {
            return Painted::Nothing;
        };
        node.subtree = (self.top_layers.len() == top_layers).then_some(painted);
        node.cached_at = self.tree.tick;
        painted.transformed(placement(node))
    }

    /// Composes `key`'s subtree; returns where it draws in its own coordinates.
    fn compose_subtree(
        &mut self,
        key: NodeKey,
        parent_transform: Transform,
        parent_opacity: f32,
        visible: Option<Rect>,
    ) -> Painted {
        let Some(node) = self.tree.nodes.get(&key) else {
            return Painted::Nothing;
        };
        if node.visual.hidden {
            return Painted::Nothing;
        }
        let values = node.values.current();
        if values.opacity <= 0.0 {
            return Painted::Nothing;
        }
        let size = node.frame.current().size;
        let placement = placement(node);
        let opacity = values.opacity.min(1.0);
        let own_shape = (Rect::new(Point::ZERO, size), values.radii);
        let backdrop = (values.backdrop_blur > 0.0).then_some(values.backdrop_blur);
        self.ops.push(CompositeOp::PushLayer(LayerProps {
            transform: placement,
            opacity,
            clip: backdrop.map(|_| own_shape),
            backdrop_blur: backdrop,
            node: Some(BoundaryId(key.0)),
        }));
        let mut painted = Painted::Nothing;
        if node.has_list {
            self.ops.push(CompositeOp::Draw(BoundaryId(key.0)));
            painted = node.painted;
        }
        if !node.children.is_empty() {
            let clip = node.visual.clip.then_some(own_shape);
            let scrolled = values.content_offset != Vector::ZERO;
            let scroll = Transform::translate(-values.content_offset);
            let clip_layer = (clip.is_some() || scrolled).then(|| {
                self.ops.push(CompositeOp::PushLayer(LayerProps {
                    clip,
                    ..LayerProps::default()
                }));
                self.ops.len() - 1
            });
            if scrolled {
                self.ops.push(CompositeOp::PushLayer(LayerProps {
                    transform: scroll,
                    ..LayerProps::default()
                }));
            }
            let children_transform = scroll.then(placement).then(parent_transform);
            let children_visible =
                children_visible(visible, placement, clip, values.content_offset);
            let children_opacity = opacity * parent_opacity;
            let mut children: Vec<NodeKey> = node.children.clone();
            let z_index = |child: &NodeKey| {
                self.tree
                    .nodes
                    .get(child)
                    .map_or(0, |child| child.visual.z_index)
            };
            if children.iter().any(|child| z_index(child) != 0) {
                children.sort_by_key(z_index);
            }
            let mut children_painted = Painted::Nothing;
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
                } else if let Some(skipped) =
                    children_visible.and_then(|visible| self.outside(child, visible))
                {
                    children_painted = children_painted.union(skipped);
                } else {
                    children_painted = children_painted.union(self.compose_node(
                        child,
                        children_transform,
                        children_opacity,
                        children_visible,
                    ));
                }
            }
            let mut children_painted = children_painted.transformed(scroll);
            if let Some((rect, radii)) = clip {
                if let Some(index) = clip_layer {
                    self.ops[index] = CompositeOp::PushLayer(LayerProps {
                        clip: needed_clip(children_painted, rect, radii),
                        ..LayerProps::default()
                    });
                }
                children_painted = children_painted.clipped(rect);
            }
            if scrolled {
                self.ops.push(CompositeOp::PopLayer);
            }
            if clip_layer.is_some() {
                self.ops.push(CompositeOp::PopLayer);
            }
            painted = painted.union(children_painted);
        }
        self.ops.push(CompositeOp::PopLayer);
        painted
    }
}

/// What of a node's content can show: what its ancestors let through, moved into the node by
/// its `placement` when that only translates, cut by its own `clip`, then moved by its scroll
/// `offset` into its children's coordinates.
fn children_visible(
    visible: Option<Rect>,
    placement: Transform,
    clip: Option<(Rect, CornerRadii)>,
    offset: Vector,
) -> Option<Rect> {
    let [a, b, c, d, x, y] = placement.matrix;
    let inherited = visible
        .filter(|_| (a, b, c, d) == (1.0, 0.0, 0.0, 1.0))
        .map(|visible| visible.translate(Vector::new(-x, -y)));
    let own = match (inherited, clip) {
        (Some(inherited), Some((rect, _))) => {
            Some(inherited.intersection(rect).unwrap_or(Rect::ZERO))
        }
        (None, Some((rect, _))) => Some(rect),
        (inherited, None) => inherited,
    };
    own.map(|rect| rect.translate(offset))
}

/// The cheapest clip that shows `children` as clipping them to `rect` with `radii` would: none
/// when they stay inside the rectangle and out of its rounded corners, a plain rectangle when
/// they only stay out of the corners (a rounded clip costs an offscreen layer).
fn needed_clip(children: Painted, rect: Rect, radii: CornerRadii) -> Option<(Rect, CornerRadii)> {
    if !radii.is_zero() && !children.clears_corners(rect, radii) {
        Some((rect, radii))
    } else if children.inside(rect) {
        None
    } else {
        Some((rect, CornerRadii::ZERO))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CARD: Rect = Rect::from_xywh(0.0, 0.0, 300.0, 200.0);
    const ROUNDED: CornerRadii = CornerRadii::uniform(12.0);

    #[test]
    fn content_clear_of_the_corners_and_inside_needs_no_clip() {
        let rows = Painted::rect(Rect::from_xywh(32.0, 32.0, 236.0, 136.0));
        assert_eq!(needed_clip(rows, CARD, ROUNDED), None);
    }

    #[test]
    fn content_clear_of_the_corners_but_overflowing_is_cut_by_a_rectangle() {
        let scrolled = Painted::rect(Rect::from_xywh(32.0, -400.0, 236.0, 900.0));
        assert_eq!(
            needed_clip(scrolled, CARD, ROUNDED),
            Some((CARD, CornerRadii::ZERO))
        );
    }

    #[test]
    fn content_in_a_corner_keeps_the_rounded_clip_even_when_inside() {
        let fill = Painted::rect(CARD);
        assert_eq!(needed_clip(fill, CARD, ROUNDED), Some((CARD, ROUNDED)));
    }
}

#[cfg(test)]
mod culling_tests {
    use std::time::Duration;

    use geometry::{Size, rgb};
    use graphics::RecordingRenderer;
    use view::{RenderOp, Visual};

    use super::*;

    const VIEWPORT: NodeKey = NodeKey(1);
    const SHOWN: NodeKey = NodeKey(2);
    const SCROLLED_AWAY: NodeKey = NodeKey(3);

    fn draws(composition: &Composition, key: NodeKey) -> bool {
        composition
            .ops
            .contains(&CompositeOp::Draw(BoundaryId(key.0)))
    }

    #[test]
    fn content_scrolled_out_of_a_clip_is_left_out_once_known_and_back_when_it_moves() {
        let mut tree = RenderTree::default();
        let mut renderer = RecordingRenderer::default();
        let row = |key, y| {
            [
                RenderOp::Frame(
                    key,
                    Rect::new(Point::new(0.0, y), Size::new(100.0, 40.0)),
                    None,
                ),
                RenderOp::Visual(
                    key,
                    Visual {
                        background: Some(rgb(0x3b82f6)),
                        ..Visual::default()
                    },
                    None,
                ),
            ]
        };
        let mut ops = vec![
            RenderOp::SetRoot(VIEWPORT),
            RenderOp::SetChildren(VIEWPORT, vec![SHOWN, SCROLLED_AWAY]),
            RenderOp::Frame(VIEWPORT, Rect::from_xywh(0.0, 0.0, 100.0, 100.0), None),
            RenderOp::Visual(
                VIEWPORT,
                Visual {
                    clip: true,
                    ..Visual::default()
                },
                None,
            ),
        ];
        ops.extend(row(SHOWN, 0.0));
        ops.extend(row(SCROLLED_AWAY, 500.0));
        tree.apply(ops, Duration::ZERO, &mut renderer);
        let (first, _) = tree.prepare(Duration::ZERO, &mut renderer);
        assert!(draws(&first, SHOWN) && draws(&first, SCROLLED_AWAY));
        let (second, _) = tree.prepare(Duration::ZERO, &mut renderer);
        assert!(draws(&second, SHOWN) && !draws(&second, SCROLLED_AWAY));
        tree.apply(row(SCROLLED_AWAY, 50.0), Duration::ZERO, &mut renderer);
        let (third, _) = tree.prepare(Duration::ZERO, &mut renderer);
        assert!(draws(&third, SCROLLED_AWAY));
    }
}
