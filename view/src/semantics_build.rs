use geometry::Rect;
use semantics::{ActionKind, Role, SemanticId, SemanticNode, SemanticTree, States};

use crate::{NodeId, handler::ListenerKind, node::NodeContent, tree::ElementTree};

impl ElementTree {
    /// Builds the semantic tree: meaningful nodes are kept, plain containers are flattened.
    pub(crate) fn build_semantics(
        &self,
        focused: Option<NodeId>,
        viewport: Rect,
    ) -> (Option<SemanticId>, Vec<SemanticNode>) {
        let Some(root) = self.root() else {
            return (None, Vec::new());
        };
        let mut nodes = Vec::new();
        let mut root_node = SemanticNode::new(SemanticId(0), Role::Window);
        root_node.bounds = viewport;
        root_node.path = "window".into();
        let mut children = Vec::new();
        self.collect(root, focused, viewport, &mut nodes, &mut children);
        root_node.children = children;
        nodes.push(root_node);
        (Some(SemanticId(0)), nodes)
    }

    fn collect(
        &self,
        id: NodeId,
        focused: Option<NodeId>,
        viewport: Rect,
        out: &mut Vec<SemanticNode>,
        siblings: &mut Vec<SemanticId>,
    ) {
        let Some(element) = self.get(id) else {
            return;
        };
        if element.visual.hidden {
            return;
        }
        let role = element.semantics.role.unwrap_or(element.role);
        let interactive = element.listeners.iter().any(|listener| {
            matches!(
                listener.kind(),
                ListenerKind::Click
                    | ListenerKind::DoubleClick
                    | ListenerKind::ContextMenu
                    | ListenerKind::LongPress
                    | ListenerKind::DragStart
                    | ListenerKind::Input
                    | ListenerKind::TextInput
            )
        });
        let meaningful = role != Role::Group
            || element.semantics.label.is_some()
            || interactive
            || element.semantics.focusable == Some(true)
            || element.style.layout.overflow == layouting::Overflow::Scroll;
        if !meaningful {
            for child in &element.children {
                self.collect(*child, focused, viewport, out, siblings);
            }
            return;
        }
        let role = if role == Role::Group && interactive {
            Role::Button
        } else {
            role
        };
        let mut node = SemanticNode::new(element.semantic_id, role);
        let named_by_content = interactive || name_from_content(role);
        node.name = element
            .semantics
            .label
            .clone()
            .or_else(|| match &element.content {
                NodeContent::Text(text) => Some(text.text.clone()),
                _ if named_by_content => self.descendant_text(id),
                _ => None,
            });
        node.value = element.semantics.value.clone();
        node.description = element.semantics.description.clone();
        node.bounds = element.geometry.window_rect;
        node.path = self.path(id);
        node.states = States {
            focused: focused == Some(id),
            focusable: element.is_focusable(),
            disabled: element.disabled,
            checked: element.semantics.checked,
            expanded: None,
            selected: false,
            secure: element.semantics.secure,
            offscreen: viewport
                .intersection(element.geometry.window_rect)
                .is_none(),
        };
        node.actions = element
            .listeners
            .iter()
            .filter_map(|listener| match listener.kind() {
                ListenerKind::Click => Some(ActionKind::Click),
                ListenerKind::DoubleClick => Some(ActionKind::DoubleClick),
                ListenerKind::ContextMenu => Some(ActionKind::ContextMenu),
                ListenerKind::LongPress => Some(ActionKind::LongPress),
                ListenerKind::DragStart | ListenerKind::Drag => Some(ActionKind::Drag),
                ListenerKind::Input | ListenerKind::TextInput => Some(ActionKind::SetValue),
                _ => None,
            })
            .collect();
        if node.states.focusable {
            node.actions.push(ActionKind::Focus);
        }
        if element.style.layout.overflow == layouting::Overflow::Scroll {
            node.actions.push(ActionKind::Scroll);
        }
        node.actions.dedup();
        let mut children = Vec::new();
        if !matches!(element.content, NodeContent::Text(_)) && !atomic(role) {
            for child in &element.children {
                self.collect(*child, focused, viewport, out, &mut children);
            }
        }
        node.children = children;
        siblings.push(node.id);
        out.push(node);
    }

    fn descendant_text(&self, id: NodeId) -> Option<String> {
        let mut parts = Vec::new();
        let mut stack = vec![id];
        while let Some(current) = stack.pop() {
            let Some(element) = self.get(current) else {
                continue;
            };
            if let NodeContent::Text(text) = &element.content {
                parts.push(text.text.clone());
            }
            stack.extend(element.children.iter().rev().copied());
        }
        (!parts.is_empty()).then(|| parts.join(" "))
    }
}

/// ARIA roles whose accessible name comes from their content when no label is set.
fn name_from_content(role: Role) -> bool {
    matches!(
        role,
        Role::Button
            | Role::Link
            | Role::MenuItem
            | Role::Tab
            | Role::Heading
            | Role::Checkbox
            | Role::Switch
            | Role::RadioButton
    )
}

/// Roles exposed as a single node: their descendants only contribute the name.
fn atomic(role: Role) -> bool {
    matches!(
        role,
        Role::Button
            | Role::Link
            | Role::MenuItem
            | Role::Tab
            | Role::TextField
            | Role::SearchField
            | Role::Switch
            | Role::Checkbox
    )
}

pub(crate) fn rebuild(
    tree: &ElementTree,
    semantic: &mut SemanticTree,
    focused: Option<NodeId>,
    viewport: Rect,
) -> semantics::SemanticDiff {
    let (root, nodes) = tree.build_semantics(focused, viewport);
    semantic.replace(root, nodes)
}
