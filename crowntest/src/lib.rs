#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod app;
mod app_icons;

pub use app::TestApp;
pub use semantics::{Query, Role, name, role, text};
