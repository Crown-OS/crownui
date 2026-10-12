#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod action;
mod node;
mod query;
mod tree;

pub use action::{ActionKind, UiAction};
pub use node::{Role, SemanticId, SemanticNode, States};
pub use query::{Query, QueryError, name, role, text};
pub use tree::{SemanticDiff, SemanticTree};
