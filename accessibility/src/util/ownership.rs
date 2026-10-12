use std::collections::HashMap;

use semantics::{SemanticId, SemanticNode, SemanticTree};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Slot {
    parent: SemanticId,
    index: usize,
}

/// The strict tree AccessKit requires, carved out of a semantic tree: every reachable node
/// is owned by exactly one parent slot, dangling, repeated and cyclic children are dropped.
pub(crate) struct Ownership {
    root: Option<SemanticId>,
    order: Vec<SemanticId>,
    slots: HashMap<SemanticId, Option<Slot>>,
}

impl Ownership {
    pub(crate) fn of(tree: &SemanticTree) -> Self {
        let mut ownership = Self {
            root: tree.root().filter(|root| tree.get(*root).is_some()),
            order: Vec::with_capacity(tree.len()),
            slots: HashMap::with_capacity(tree.len()),
        };
        let Some(root) = ownership.root else {
            return ownership;
        };
        ownership.slots.insert(root, None);
        let mut pending = vec![root];
        while let Some(id) = pending.pop() {
            ownership.order.push(id);
            let Some(node) = tree.get(id) else {
                continue;
            };
            let first_child = pending.len();
            for (index, &child) in node.children.iter().enumerate() {
                if tree.get(child).is_some() && !ownership.slots.contains_key(&child) {
                    ownership
                        .slots
                        .insert(child, Some(Slot { parent: id, index }));
                    pending.push(child);
                }
            }
            pending[first_child..].reverse();
        }
        ownership
    }

    pub(crate) fn root(&self) -> Option<SemanticId> {
        self.root
    }

    pub(crate) fn reading_order(&self) -> &[SemanticId] {
        &self.order
    }

    pub(crate) fn contains(&self, id: SemanticId) -> bool {
        self.slots.contains_key(&id)
    }

    pub(crate) fn has_detached_nodes(&self, tree: &SemanticTree) -> bool {
        self.order.len() < tree.len()
    }

    pub(crate) fn owned_children<'a>(
        &'a self,
        node: &'a SemanticNode,
    ) -> impl Iterator<Item = SemanticId> + 'a {
        node.children
            .iter()
            .enumerate()
            .filter(move |&(index, child)| {
                self.slots.get(child)
                    == Some(&Some(Slot {
                        parent: node.id,
                        index,
                    }))
            })
            .map(|(_, &child)| child)
    }
}

#[cfg(test)]
mod tests {
    use semantics::Role;

    use super::*;

    fn tree_with_children(children: &[(u64, &[u64])]) -> SemanticTree {
        let nodes = children.iter().map(|&(id, kids)| {
            let mut node = SemanticNode::new(SemanticId(id), Role::Group);
            node.children = kids.iter().copied().map(SemanticId).collect();
            node
        });
        let mut tree = SemanticTree::new();
        tree.replace(Some(SemanticId(1)), nodes);
        tree
    }

    #[test]
    fn repeated_dangling_and_cyclic_children_are_owned_once() {
        let tree = tree_with_children(&[(1, &[2, 3, 2, 99]), (2, &[1, 3]), (3, &[]), (4, &[])]);
        let ownership = Ownership::of(&tree);
        let owned = |id| {
            ownership
                .owned_children(tree.get(SemanticId(id)).expect("node exists"))
                .collect::<Vec<_>>()
        };
        assert_eq!(owned(1), vec![SemanticId(2), SemanticId(3)]);
        assert!(owned(2).is_empty());
        assert_eq!(
            ownership.reading_order(),
            &[SemanticId(1), SemanticId(2), SemanticId(3)]
        );
        assert!(ownership.has_detached_nodes(&tree));
    }
}
