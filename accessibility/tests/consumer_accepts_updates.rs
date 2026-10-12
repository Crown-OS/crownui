use accessibility::{diff_to_tree_update, to_tree_update};
use accesskit_consumer::{NodeRef, Tree, TreeChangeHandler};
use semantics::{Role, SemanticId, SemanticNode, SemanticTree};

#[derive(Default)]
struct ChangeCounter {
    added: usize,
    updated: usize,
    removed: usize,
}

impl TreeChangeHandler for ChangeCounter {
    fn node_added(&mut self, _: &NodeRef) {
        self.added += 1;
    }
    fn node_updated(&mut self, _: &NodeRef, _: &NodeRef) {
        self.updated += 1;
    }
    fn focus_moved(&mut self, _: Option<&NodeRef>, _: Option<&NodeRef>) {}
    fn node_removed(&mut self, _: &NodeRef) {
        self.removed += 1;
    }
}

fn button(id: u64, name: &str) -> SemanticNode {
    let mut node = SemanticNode::new(SemanticId(id), Role::Button);
    node.name = Some(name.into());
    node
}

fn window(children: &[u64]) -> SemanticNode {
    let mut node = SemanticNode::new(SemanticId(1), Role::Window);
    node.children = children.iter().copied().map(SemanticId).collect();
    node
}

#[test]
fn incremental_updates_apply_cleanly_to_an_accesskit_tree() {
    let mut semantic_tree = SemanticTree::new();
    semantic_tree.replace(
        Some(SemanticId(1)),
        [window(&[2, 3]), button(2, "Save"), button(3, "Cancel")],
    );
    let mut accesskit_tree = Tree::new(to_tree_update(&semantic_tree, Some(SemanticId(2))), true);

    let diff = semantic_tree.replace(
        Some(SemanticId(1)),
        [window(&[2, 4]), button(2, "Save As"), button(4, "Help")],
    );
    let mut changes = ChangeCounter::default();
    accesskit_tree.update_and_process_changes(
        diff_to_tree_update(&diff, &semantic_tree, Some(SemanticId(4))),
        &mut changes,
    );

    assert_eq!((changes.added, changes.removed), (1, 1));
    assert!(changes.updated >= 2);
    let state = accesskit_tree.state();
    assert_eq!(state.root().children().count(), 2);
    assert_eq!(
        state.focus().and_then(|node| node.label()).as_deref(),
        Some("Help")
    );
}
