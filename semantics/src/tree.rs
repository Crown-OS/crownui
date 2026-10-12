use std::{collections::BTreeMap, fmt};

use crate::{SemanticId, SemanticNode};

/// A complete semantic tree.
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SemanticTree {
    nodes: BTreeMap<SemanticId, SemanticNode>,
    root: Option<SemanticId>,
    revision: u64,
}

/// What changed between two trees.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SemanticDiff {
    /// New or changed nodes.
    pub updated: Vec<SemanticNode>,
    /// Removed node ids.
    pub removed: Vec<SemanticId>,
    /// The new root.
    pub root: Option<SemanticId>,
    /// The new revision.
    pub revision: u64,
}

impl SemanticTree {
    /// An empty tree.
    pub fn new() -> Self {
        Self::default()
    }

    /// The root node's id.
    pub fn root(&self) -> Option<SemanticId> {
        self.root
    }

    /// Increments on every applied change.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Looks up a node.
    pub fn get(&self, id: SemanticId) -> Option<&SemanticNode> {
        self.nodes.get(&id)
    }

    /// Number of nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the tree is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Nodes in depth-first reading order from the root.
    pub fn iter(&self) -> impl Iterator<Item = &SemanticNode> + '_ {
        let mut stack: Vec<SemanticId> = self.root.into_iter().collect();
        std::iter::from_fn(move || {
            let node = self.nodes.get(&stack.pop()?)?;
            stack.extend(node.children.iter().rev());
            Some(node)
        })
    }

    /// Replaces the whole tree, bumping the revision when anything changed.
    pub fn replace(&mut self, root: Option<SemanticId>, nodes: impl IntoIterator<Item = SemanticNode>) -> SemanticDiff {
        let next: BTreeMap<_, _> = nodes.into_iter().map(|node| (node.id, node)).collect();
        let diff = SemanticDiff {
            updated: next
                .values()
                .filter(|node| self.nodes.get(&node.id) != Some(node))
                .cloned()
                .collect(),
            removed: self.nodes.keys().filter(|id| !next.contains_key(id)).copied().collect(),
            root,
            revision: self.revision + 1,
        };
        if !diff.updated.is_empty() || !diff.removed.is_empty() || root != self.root {
            self.revision += 1;
        }
        self.nodes = next;
        self.root = root;
        SemanticDiff {
            revision: self.revision,
            ..diff
        }
    }

    /// Applies a diff produced by another tree's [`replace`](Self::replace).
    pub fn apply(&mut self, diff: &SemanticDiff) {
        for id in &diff.removed {
            self.nodes.remove(id);
        }
        for node in &diff.updated {
            self.nodes.insert(node.id, node.clone());
        }
        self.root = diff.root;
        self.revision = diff.revision;
    }

    /// The parent of `id`.
    pub fn parent_of(&self, id: SemanticId) -> Option<SemanticId> {
        self.nodes
            .values()
            .find(|node| node.children.contains(&id))
            .map(|node| node.id)
    }
}

impl fmt::Display for SemanticTree {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn write_node(tree: &SemanticTree, id: SemanticId, depth: usize, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            let Some(node) = tree.get(id) else {
                return Ok(());
            };
            write!(formatter, "{:indent$}{}", "", node.role, indent = depth * 2)?;
            if let Some(name) = &node.name {
                write!(formatter, " {name:?}")?;
            }
            match (node.states.secure, &node.value) {
                (true, Some(_)) => write!(formatter, " value=<secure>")?,
                (false, Some(value)) => write!(formatter, " value={value:?}")?,
                _ => {}
            }
            let states = &node.states;
            let flags = [
                (states.focused, "focused"),
                (states.disabled, "disabled"),
                (states.selected, "selected"),
                (states.checked == Some(true), "checked"),
                (states.expanded == Some(true), "expanded"),
                (states.offscreen, "offscreen"),
            ];
            let active: Vec<_> = flags.iter().filter(|(on, _)| *on).map(|(_, label)| *label).collect();
            if !active.is_empty() {
                write!(formatter, " [{}]", active.join(", "))?;
            }
            let bounds = node.bounds;
            writeln!(
                formatter,
                " @{},{} {}x{}",
                geometry::Length(bounds.min_x()),
                geometry::Length(bounds.min_y()),
                geometry::Length(bounds.size.width),
                geometry::Length(bounds.size.height)
            )?;
            for child in &node.children {
                write_node(tree, *child, depth + 1, formatter)?;
            }
            Ok(())
        }
        match self.root {
            Some(root) => write_node(self, root, 0, formatter),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use geometry::Rect;

    use super::*;
    use crate::{QueryError, Role, role, text};

    fn sample() -> SemanticTree {
        let mut window = SemanticNode::new(SemanticId(1), Role::Window);
        window.children = vec![SemanticId(2), SemanticId(3), SemanticId(4)];
        let mut save = SemanticNode::new(SemanticId(2), Role::Button);
        save.name = Some("Save".into());
        save.bounds = Rect::from_xywh(10.0, 10.0, 80.0, 32.0);
        let mut cancel = SemanticNode::new(SemanticId(3), Role::Button);
        cancel.name = Some("Cancel".into());
        let mut password = SemanticNode::new(SemanticId(4), Role::TextField);
        password.name = Some("Password".into());
        password.value = Some("hunter2".into());
        password.states.secure = true;
        let mut tree = SemanticTree::new();
        tree.replace(Some(SemanticId(1)), [window, save, cancel, password]);
        tree
    }

    #[test]
    fn trees_print_as_indented_text_without_secrets() {
        assert_eq!(
            sample().to_string(),
            "window @0,0 0x0\n  button \"Save\" @10,10 80x32\n  button \"Cancel\" @0,0 0x0\n  text_field \"Password\" value=<secure> @0,0 0x0\n"
        );
    }

    #[test]
    fn queries_find_one_node_or_explain_why_not() {
        let tree = sample();
        assert_eq!(role(Role::Button).name("Save").find_one(&tree), Ok(SemanticId(2)));
        assert!(matches!(role(Role::Button).find_one(&tree), Err(QueryError::Ambiguous { count: 2, .. })));
        assert!(matches!(text("hunter2").find_one(&tree), Err(QueryError::NotFound { .. })));
    }

    #[test]
    fn diffs_carry_only_changes_and_bump_the_revision() {
        let mut tree = sample();
        let before = tree.revision();
        let mut nodes: Vec<_> = tree.iter().cloned().collect();
        nodes.retain(|node| node.id != SemanticId(3));
        if let Some(window) = nodes.iter_mut().find(|node| node.id == SemanticId(1)) {
            window.children.retain(|child| *child != SemanticId(3));
        }
        let diff = tree.replace(Some(SemanticId(1)), nodes);
        assert_eq!(diff.removed, vec![SemanticId(3)]);
        assert_eq!(diff.updated.len(), 1);
        assert_eq!(tree.revision(), before + 1);
        let mut mirror = SemanticTree::new();
        mirror.apply(&diff);
        assert_eq!(mirror.revision(), tree.revision());
    }
}
