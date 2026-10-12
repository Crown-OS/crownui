#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod action;
mod adapter;
mod capability;
mod event;
mod exporter;
mod handlers;
mod node;
mod request;
mod role;
mod toolkit;
mod util;

pub use action::AccessibilityAction;
pub use adapter::AccessibilityAdapter;
pub use event::{AccessibilityEvent, AccessibilityEvents};
pub use exporter::{TreeExporter, diff_to_tree_update, to_tree_update};
pub use node::SYNTHETIC_WINDOW_ID;
pub use request::{SCROLL_LINE_STEP, SCROLL_PAGE_STEP, action_request_to_ui};
pub use toolkit::Toolkit;

pub use accesskit;
