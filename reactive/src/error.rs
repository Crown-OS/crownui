use crate::ScopeId;

/// Why a flush stopped early.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FlushError {
    /// Effects kept writing signals that re-triggered them.
    #[error(
        "reactive cycle: still dirty after {passes} passes (an effect writes a signal it reads)"
    )]
    Cycle {
        /// Passes run before giving up.
        passes: usize,
    },
}

/// An observer that panicked and was quarantined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObserverPanic {
    /// Scope the observer belonged to.
    pub scope: ScopeId,
    /// The panic payload, when it was a string.
    pub message: String,
}
