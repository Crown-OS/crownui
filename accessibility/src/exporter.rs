use accesskit::{Affine, Node, NodeId, TreeId, TreeInfo, TreeUpdate};
use semantics::{SemanticDiff, SemanticId, SemanticNode, SemanticTree};

use crate::{
    Toolkit,
    node::{SYNTHETIC_WINDOW_ID, node_id, to_accesskit_node},
    util::ownership::Ownership,
};

const UNSCALED: f64 = 1.0;

/// Converts semantic trees and diffs into AccessKit tree updates.
///
/// Bounds stay in logical window pixels; a scale factor other than 1 is applied as a
/// transform on the root node, which AccessKit propagates to every descendant.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeExporter {
    window_label: Option<String>,
    toolkit: Toolkit,
    scale_factor: f64,
}

impl Default for TreeExporter {
    fn default() -> Self {
        Self {
            window_label: None,
            toolkit: Toolkit::default(),
            scale_factor: UNSCALED,
        }
    }
}

/// A full AccessKit update for `tree`, with default exporter settings.
pub fn to_tree_update(tree: &SemanticTree, focus: Option<SemanticId>) -> TreeUpdate {
    TreeExporter::default().full(tree, focus)
}

/// An incremental AccessKit update carrying only the nodes `diff` changed, with default
/// exporter settings. See [`TreeExporter::incremental`].
pub fn diff_to_tree_update(
    diff: &SemanticDiff,
    tree_after: &SemanticTree,
    focus: Option<SemanticId>,
) -> TreeUpdate {
    TreeExporter::default().incremental(diff, tree_after, focus)
}

impl TreeExporter {
    /// An exporter with no window label, the CrownUI toolkit and no scaling.
    pub fn new() -> Self {
        Self::default()
    }

    /// Names the root window when its semantic node has no name.
    pub fn window_label(mut self, label: impl Into<String>) -> Self {
        self.window_label = Some(label.into());
        self
    }

    /// Reports `toolkit` in every update.
    pub fn toolkit(mut self, toolkit: Toolkit) -> Self {
        self.toolkit = toolkit;
        self
    }

    /// Scales logical bounds to the physical pixels AccessKit expects.
    pub fn scale_factor(mut self, scale_factor: f64) -> Self {
        self.scale_factor = scale_factor;
        self
    }

    pub(crate) fn set_scale_factor(&mut self, scale_factor: f64) -> bool {
        let changed = self.scale_factor != scale_factor;
        self.scale_factor = scale_factor;
        changed
    }

    /// A full update describing every node reachable from the root.
    ///
    /// A tree without a root exports as an empty window with id [`SYNTHETIC_WINDOW_ID`].
    /// Focus falls back to the root when `focus` is `None` or not in the tree.
    pub fn full(&self, tree: &SemanticTree, focus: Option<SemanticId>) -> TreeUpdate {
        self.full_with(tree, &Ownership::of(tree), focus)
    }

    /// An update carrying the root plus the reachable nodes `diff` added or changed;
    /// AccessKit drops nodes no parent references anymore.
    ///
    /// Falls back to a full update while `tree_after` holds nodes unreachable from the
    /// root, since AccessKit rejects updates naming nodes it cannot place. A node that
    /// becomes reachable later without changing never appears in a diff, so callers that
    /// detach and reattach subtrees should send a full update on the frame after the
    /// last detached node disappears; [`AccessibilityAdapter`](crate::AccessibilityAdapter)
    /// does this automatically.
    pub fn incremental(
        &self,
        diff: &SemanticDiff,
        tree_after: &SemanticTree,
        focus: Option<SemanticId>,
    ) -> TreeUpdate {
        let ownership = Ownership::of(tree_after);
        if ownership.has_detached_nodes(tree_after) {
            self.full_with(tree_after, &ownership, focus)
        } else {
            self.incremental_with(diff, tree_after, &ownership, focus)
        }
    }

    pub(crate) fn full_with(
        &self,
        tree: &SemanticTree,
        ownership: &Ownership,
        focus: Option<SemanticId>,
    ) -> TreeUpdate {
        let nodes = ownership
            .reading_order()
            .iter()
            .filter_map(|&id| tree.get(id));
        self.assemble(nodes, ownership, focus)
    }

    pub(crate) fn incremental_with(
        &self,
        diff: &SemanticDiff,
        tree_after: &SemanticTree,
        ownership: &Ownership,
        focus: Option<SemanticId>,
    ) -> TreeUpdate {
        let root = ownership.root();
        let changed = diff
            .updated
            .iter()
            .map(|node| node.id)
            .filter(|&id| Some(id) != root && ownership.contains(id));
        let nodes = root
            .into_iter()
            .chain(changed)
            .filter_map(|id| tree_after.get(id));
        self.assemble(nodes, ownership, focus)
    }

    fn assemble<'tree>(
        &self,
        nodes: impl Iterator<Item = &'tree SemanticNode>,
        ownership: &Ownership,
        focus: Option<SemanticId>,
    ) -> TreeUpdate {
        let root = ownership.root().map_or(SYNTHETIC_WINDOW_ID, node_id);
        let synthetic_window = ownership.root().is_none().then(|| {
            (
                SYNTHETIC_WINDOW_ID,
                self.decorate_root(Node::new(accesskit::Role::Window)),
            )
        });
        let exported = synthetic_window
            .into_iter()
            .chain(nodes.map(|node| self.export(node, ownership)))
            .collect();
        TreeUpdate {
            nodes: exported,
            tree: Some(TreeInfo {
                root,
                toolkit_name: Some(self.toolkit.name.to_string()),
                toolkit_version: Some(self.toolkit.version.to_string()),
            }),
            tree_id: TreeId::ROOT,
            focus: focus
                .filter(|&id| ownership.contains(id))
                .map_or(root, node_id),
        }
    }

    fn export(&self, node: &SemanticNode, ownership: &Ownership) -> (NodeId, Node) {
        let exported = to_accesskit_node(node, ownership.owned_children(node));
        let exported = if ownership.root() == Some(node.id) {
            self.decorate_root(exported)
        } else {
            exported
        };
        (node_id(node.id), exported)
    }

    fn decorate_root(&self, mut root: Node) -> Node {
        if root.label().is_none()
            && let Some(label) = self.window_label.as_deref()
        {
            root.set_label(label);
        }
        if self.scale_factor != UNSCALED {
            root.set_transform(Affine::scale(self.scale_factor));
        }
        root
    }
}

#[cfg(test)]
mod tests {
    use geometry::Rect;
    use semantics::Role;

    use super::*;

    fn window_with_buttons() -> SemanticTree {
        let mut window = SemanticNode::new(SemanticId(1), Role::Window);
        window.children = vec![SemanticId(2), SemanticId(3)];
        window.bounds = Rect::from_xywh(0.0, 0.0, 400.0, 300.0);
        let mut save = SemanticNode::new(SemanticId(2), Role::Button);
        save.name = Some("Save".into());
        save.bounds = Rect::from_xywh(10.0, 20.0, 80.0, 32.0);
        let mut title = SemanticNode::new(SemanticId(3), Role::Text);
        title.name = Some("Untitled".into());
        let mut tree = SemanticTree::new();
        tree.replace(Some(SemanticId(1)), [window, save, title]);
        tree
    }

    fn exported(update: &TreeUpdate, id: u64) -> &Node {
        update
            .nodes
            .iter()
            .find(|(node_id, _)| *node_id == NodeId(id))
            .map(|(_, node)| node)
            .unwrap_or_else(|| panic!("node {id} missing from update"))
    }

    #[test]
    fn full_update_describes_root_children_labels_and_bounds() {
        let update = to_tree_update(&window_with_buttons(), Some(SemanticId(2)));
        let info = update.tree.as_ref().expect("full updates carry tree info");
        assert_eq!(info.root, NodeId(1));
        assert_eq!(info.toolkit_name.as_deref(), Some("CrownUI"));
        assert_eq!(update.focus, NodeId(2));
        assert_eq!(update.nodes.len(), 3);

        let window = exported(&update, 1);
        assert_eq!(window.role(), accesskit::Role::Window);
        assert_eq!(window.children(), &[NodeId(2), NodeId(3)]);

        let save = exported(&update, 2);
        assert_eq!(save.label(), Some("Save"));
        assert_eq!(
            save.bounds(),
            Some(accesskit::Rect::new(10.0, 20.0, 90.0, 52.0))
        );

        let title = exported(&update, 3);
        assert_eq!(title.role(), accesskit::Role::Label);
        assert_eq!(title.value(), Some("Untitled"));
    }

    #[test]
    fn incremental_update_carries_only_the_root_and_changed_nodes() {
        let mut tree = window_with_buttons();
        let mut nodes: Vec<_> = tree.iter().cloned().collect();
        if let Some(save) = nodes.iter_mut().find(|node| node.id == SemanticId(2)) {
            save.name = Some("Save As".into());
        }
        let diff = tree.replace(Some(SemanticId(1)), nodes);
        let update = diff_to_tree_update(&diff, &tree, Some(SemanticId(99)));
        let ids: Vec<_> = update.nodes.iter().map(|(id, _)| *id).collect();
        assert_eq!(ids, vec![NodeId(1), NodeId(2)]);
        assert_eq!(exported(&update, 2).label(), Some("Save As"));
        assert_eq!(update.focus, NodeId(1));
    }

    #[test]
    fn detached_nodes_force_a_full_update_without_them() {
        let mut tree = window_with_buttons();
        let mut nodes: Vec<_> = tree.iter().cloned().collect();
        nodes.push(SemanticNode::new(SemanticId(50), Role::Button));
        let diff = tree.replace(Some(SemanticId(1)), nodes);
        let update = diff_to_tree_update(&diff, &tree, None);
        assert_eq!(update.nodes.len(), 3);
        assert!(update.nodes.iter().all(|(id, _)| *id != NodeId(50)));
    }

    #[test]
    fn rootless_trees_export_a_labelled_synthetic_window() {
        let update = TreeExporter::new()
            .window_label("Notes")
            .scale_factor(2.0)
            .full(&SemanticTree::new(), None);
        assert_eq!(update.focus, SYNTHETIC_WINDOW_ID);
        let window = exported(&update, SYNTHETIC_WINDOW_ID.0);
        assert_eq!(window.label(), Some("Notes"));
        assert_eq!(window.transform(), Some(&Affine::scale(2.0)));
    }
}
