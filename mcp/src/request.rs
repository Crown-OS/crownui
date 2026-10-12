use std::fmt;

use inspect::InspectSnapshot;
use semantics::{SemanticId, SemanticTree, UiAction};
use tokio::sync::oneshot;

use crate::ActError;

/// Answers one [`McpRequest`]; dropping it unanswered reports an error to the agent.
pub struct Responder<T>(oneshot::Sender<T>);

impl<T> Responder<T> {
    pub(crate) fn new() -> (Self, oneshot::Receiver<T>) {
        let (sender, receiver) = oneshot::channel();
        (Self(sender), receiver)
    }

    /// Sends the answer; a no-op when the agent has already given up.
    pub fn respond(self, value: T) {
        let _ = self.0.send(value);
    }

    /// Whether the agent stopped waiting (timeout or disconnect).
    pub fn is_abandoned(&self) -> bool {
        self.0.is_closed()
    }
}

impl<T> fmt::Debug for Responder<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Responder")
    }
}

/// The semantic tree at one revision.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeSnapshot {
    /// Revision agents must quote in `ui_act`.
    pub revision: u64,
    /// The tree.
    pub tree: SemanticTree,
}

impl From<SemanticTree> for TreeSnapshot {
    fn from(tree: SemanticTree) -> Self {
        Self {
            revision: tree.revision(),
            tree,
        }
    }
}

/// A successfully dispatched action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActOutcome {
    /// Tree revision after dispatching the action.
    pub revision: u64,
}

/// A PNG capture of the app window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Screenshot {
    /// Encoded PNG bytes.
    pub png: Vec<u8>,
}

/// Work the server needs from the UI thread, drained with
/// [`McpHandle::pending_requests`](crate::McpHandle::pending_requests).
#[derive(Debug)]
pub enum McpRequest {
    /// Return the current semantic tree.
    Tree {
        /// Where to send it.
        reply: Responder<TreeSnapshot>,
    },
    /// Perform `action` on node `id`, if the tree is still at `revision`.
    Act {
        /// Target node.
        id: SemanticId,
        /// Revision the agent read the id from.
        revision: u64,
        /// What to do.
        action: UiAction,
        /// Where to send the outcome; see [`validate_action`](crate::validate_action).
        reply: Responder<Result<ActOutcome, ActError>>,
    },
    /// Run an app tool registered with [`McpHandle::set_tools`](crate::McpHandle::set_tools).
    CallTool {
        /// Tool name.
        name: String,
        /// Arguments object, without the `confirm` flag of destructive tools.
        arguments: serde_json::Value,
        /// Where to send the result or an error message.
        reply: Responder<Result<serde_json::Value, String>>,
    },
    /// Return an introspection snapshot.
    Inspect {
        /// Where to send it.
        reply: Responder<InspectSnapshot>,
    },
    /// Capture the window.
    Screenshot {
        /// Where to send the capture, or why there is none.
        reply: Responder<Result<Screenshot, String>>,
    },
}
