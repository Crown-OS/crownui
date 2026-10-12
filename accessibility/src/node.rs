use accesskit::{Action, Node, NodeId, Toggled};
use semantics::{Role, SemanticId, SemanticNode};

use crate::{capability::accesskit_actions, role::accesskit_role, util::rect::to_accesskit_rect};

/// The AccessKit id of the window node exported when the semantic tree has no root.
pub const SYNTHETIC_WINDOW_ID: NodeId = NodeId(u64::MAX);

pub(crate) fn node_id(id: SemanticId) -> NodeId {
    NodeId(id.0)
}

pub(crate) fn to_accesskit_node(
    node: &SemanticNode,
    children: impl Iterator<Item = SemanticId>,
) -> Node {
    let mut exported = Node::new(accesskit_role(node.role, &node.states));
    exported.set_children(children.map(node_id).collect::<Vec<_>>());
    exported.set_bounds(to_accesskit_rect(node.bounds));
    export_text(&mut exported, node);
    export_states(&mut exported, node);
    export_actions(&mut exported, node);
    exported
}

fn export_text(exported: &mut Node, node: &SemanticNode) {
    let name = node.name.as_deref();
    let value = node.exposed_value();
    if exported.role() == accesskit::Role::Label {
        if let Some(text) = name.or(value) {
            exported.set_value(text);
        }
    } else {
        if let Some(name) = name {
            exported.set_label(name);
        }
        if let Some(value) = value {
            exported.set_value(value);
        }
    }
    if let Some(description) = node.description.as_deref() {
        exported.set_description(description);
    }
}

fn export_states(exported: &mut Node, node: &SemanticNode) {
    let states = &node.states;
    if states.disabled {
        exported.set_disabled();
    }
    if let Some(checked) = states.checked {
        exported.set_toggled(if checked {
            Toggled::True
        } else {
            Toggled::False
        });
    }
    if let Some(expanded) = states.expanded {
        exported.set_expanded(expanded);
    }
    if states.selected || node.role == Role::Tab {
        exported.set_selected(states.selected);
    }
    if node.role == Role::ScrollView {
        exported.set_clips_children();
    }
}

fn export_actions(exported: &mut Node, node: &SemanticNode) {
    node.actions
        .iter()
        .flat_map(|&kind| accesskit_actions(kind))
        .for_each(|&action| exported.add_action(action));
    if node.states.focusable {
        exported.add_action(Action::Focus);
    }
    if node.states.offscreen {
        exported.add_action(Action::ScrollIntoView);
    }
}

#[cfg(test)]
mod tests {
    use semantics::ActionKind;

    use super::*;

    #[test]
    fn secure_values_are_never_exported() {
        let mut password = SemanticNode::new(SemanticId(7), Role::TextField);
        password.name = Some("Password".into());
        password.value = Some("hunter2".into());
        password.states.secure = true;
        let exported = to_accesskit_node(&password, std::iter::empty());
        assert_eq!(exported.role(), accesskit::Role::PasswordInput);
        assert_eq!(exported.label(), Some("Password"));
        assert_eq!(exported.value(), None);

        password.role = Role::Text;
        password.name = None;
        assert_eq!(
            to_accesskit_node(&password, std::iter::empty()).value(),
            None
        );
    }

    #[test]
    fn states_and_capabilities_become_flags_and_actions() {
        let mut checkbox = SemanticNode::new(SemanticId(3), Role::Checkbox);
        checkbox.states.checked = Some(true);
        checkbox.states.disabled = true;
        checkbox.states.focusable = true;
        checkbox.states.offscreen = true;
        checkbox.actions = vec![ActionKind::Click, ActionKind::Scroll];
        let exported = to_accesskit_node(&checkbox, std::iter::empty());
        assert_eq!(exported.toggled(), Some(Toggled::True));
        assert!(exported.is_disabled());
        assert!(!exported.is_hidden());
        for action in [
            Action::Click,
            Action::Focus,
            Action::ScrollIntoView,
            Action::ScrollDown,
            Action::ScrollUp,
        ] {
            assert!(exported.supports_action(action), "{action:?}");
        }
        assert_eq!(exported.is_selected(), None);
    }
}
