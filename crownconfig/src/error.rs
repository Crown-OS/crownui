use std::io;

use thiserror::Error;

/// Why a config section could not be written.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// The section's file could not be created or replaced.
    #[error("CUI0601: writing config section `{section}`: {source}")]
    Write {
        /// The section's name, which is also its file stem.
        section: String,
        /// The underlying I/O failure.
        #[source]
        source: io::Error,
    },
}
