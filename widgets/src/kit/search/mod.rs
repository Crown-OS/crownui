mod kind;
mod metrics;
mod options;
mod picker;
mod popup;
mod row;
mod state;
mod suggestion;

pub use kind::{SearchKind, search};
pub use options::SearchOptions;
pub use suggestion::{Suggestion, suggestion};

/// What an empty search bar says.
pub const DEFAULT_PLACEHOLDER: &str = "Search…";
