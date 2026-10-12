use geometry::Size;
use layouting::{LayoutEngine, LayoutStyle, MeasureInput, NodeLayout};
use taffy::{TaffyTree, compute_leaf_layout};

use crate::{
    TaffyNode,
    convert::{measure_input, node_layout, root_available_space, taffy_size, taffy_style},
    measured_leaf::MeasuredLeaf,
    util::node_result::debug_assert_node_ok,
};

/// A [`LayoutEngine`] running taffy's flexbox and grid algorithms.
///
/// Rounding is disabled: layouts stay in fractional logical pixels so the framework can snap
/// them to device pixels at the actual scale factor.
pub struct TaffyEngine {
    tree: TaffyTree<MeasuredLeaf>,
}

impl TaffyEngine {
    /// Creates an empty engine.
    pub fn new() -> Self {
        let mut tree = TaffyTree::new();
        tree.disable_rounding();
        Self { tree }
    }

    /// Number of live nodes.
    pub fn node_count(&self) -> usize {
        self.tree.total_node_count()
    }
}

impl Default for TaffyEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutEngine for TaffyEngine {
    type Node = TaffyNode;

    fn create_node(&mut self, style: &LayoutStyle, measured: bool) -> TaffyNode {
        let style = taffy_style(style);
        let created = if measured {
            self.tree.new_leaf_with_context(style, MeasuredLeaf)
        } else {
            self.tree.new_leaf(style)
        };
        match created {
            Ok(node_id) => TaffyNode(node_id),
            Err(error) => unreachable!("taffy leaf insertion is infallible: {error}"),
        }
    }

    fn set_style(&mut self, node: TaffyNode, style: &LayoutStyle) {
        debug_assert_node_ok(self.tree.set_style(node.0, taffy_style(style)));
    }

    fn set_children(&mut self, node: TaffyNode, children: &[TaffyNode]) {
        let child_ids: Vec<_> = children.iter().map(|child| child.0).collect();
        debug_assert_node_ok(self.tree.set_children(node.0, &child_ids));
    }

    fn remove_node(&mut self, node: TaffyNode) {
        debug_assert_node_ok(self.tree.set_node_context(node.0, None));
        debug_assert_node_ok(self.tree.remove(node.0));
    }

    fn mark_dirty(&mut self, node: TaffyNode) {
        debug_assert_node_ok(self.tree.mark_dirty(node.0));
    }

    fn compute_layout<M>(&mut self, root: TaffyNode, available: Size, mut measure: M)
    where
        M: FnMut(TaffyNode, MeasureInput) -> Size,
    {
        let computed = self.tree.compute_layout_with_measure(
            root.0,
            root_available_space(available),
            |inputs, node_id, context, style| {
                compute_leaf_layout(
                    inputs,
                    style,
                    |_, _| 0.0,
                    |known, available_space| match context {
                        Some(MeasuredLeaf) => taffy_size(measure(
                            TaffyNode(node_id),
                            measure_input(known, available_space),
                        )),
                        None => taffy::Size::ZERO,
                    },
                )
            },
        );
        debug_assert_node_ok(computed);
    }

    fn layout(&self, node: TaffyNode) -> NodeLayout {
        self.tree
            .layout(node.0)
            .map(node_layout)
            .unwrap_or_default()
    }
}
