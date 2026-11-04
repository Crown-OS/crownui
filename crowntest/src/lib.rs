#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod app;
mod app_icons;
mod clipboard;
mod drag;

pub use app::TestApp;
pub use semantics::{Query, Role, name, role, text};
