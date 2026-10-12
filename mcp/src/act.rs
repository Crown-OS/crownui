use semantics::{Role, SemanticId, SemanticNode, SemanticTree, UiAction};

/// Why an agent's action was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ActError {
    /// The agent read its ids from an older tree.
    #[error("stale revision: the tree is now at revision {current}; read ui_tree again and retry")]
    StaleRevision {
        /// The current revision.
        current: u64,
    },
    /// No node has this id.
    #[error("no node with id {}", id.0)]
    UnknownNode {
        /// The requested id.
        id: SemanticId,
    },
    /// The node does not advertise the capability the action needs.
    #[error("{action} is not supported by this {role}")]
    Unsupported {
        /// Action name.
        action: &'static str,
        /// Role of the target.
        role: Role,
    },
    /// The node is disabled.
    #[error("the node is disabled")]
    Disabled,
    /// The node holds a secret that agents may not edit.
    #[error("the node holds a secret; agents may not type into it")]
    Secure,
    /// The app wants the user to confirm this action first.
    #[error("the app requires the user to confirm this action")]
    ConfirmationRequired,
}

/// Checks `action` against `tree` the way agents expect: revision first, then the
/// node's existence, state and capabilities. Returns the target node.
pub fn validate_action<'tree>(
    tree: &'tree SemanticTree,
    id: SemanticId,
    revision: u64,
    action: &UiAction,
) -> Result<&'tree SemanticNode, ActError> {
    if revision != tree.revision() {
        return Err(ActError::StaleRevision {
            current: tree.revision(),
        });
    }
    let node = tree.get(id).ok_or(ActError::UnknownNode { id })?;
    if node.states.disabled {
        return Err(ActError::Disabled);
    }
    if node.states.secure
        && matches!(
            action,
            UiAction::TypeText { .. } | UiAction::PressKey { .. }
        )
    {
        return Err(ActError::Secure);
    }
    match action.required_capability() {
        Some(capability) if !node.supports(capability) => Err(ActError::Unsupported {
            action: action.name(),
            role: node.role,
        }),
        _ => Ok(node),
    }
}
