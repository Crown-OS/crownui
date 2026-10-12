#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod app;

pub use app::TestApp;
pub use semantics::{Query, Role, name, role, text};
