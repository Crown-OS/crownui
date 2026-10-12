#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod error;
mod handles;
mod runtime;
mod scope;

pub use error::{FlushError, ObserverPanic};
pub use handles::{EffectId, Memo, Signal, Store};
pub use runtime::{FlushStats, Runtime};
pub use scope::ScopeId;

#[cfg(test)]
mod tests;
