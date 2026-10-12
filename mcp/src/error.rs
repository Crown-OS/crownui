use std::{io, path::PathBuf};

/// Why the MCP server could not start.
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    /// The socket directory could not be created or secured.
    #[error("cannot prepare socket directory {path}: {source}")]
    SocketDir {
        /// The directory.
        path: PathBuf,
        /// The underlying error.
        source: io::Error,
    },
    /// The socket directory belongs to another user.
    #[error("socket directory {path} is owned by another user")]
    ForeignSocketDir {
        /// The directory.
        path: PathBuf,
    },
    /// The socket could not be bound.
    #[error("cannot bind MCP socket {path}: {source}")]
    Bind {
        /// The socket path.
        path: PathBuf,
        /// The underlying error.
        source: io::Error,
    },
    /// The server thread or its runtime could not start.
    #[error("cannot start the MCP server thread: {0}")]
    Runtime(#[from] io::Error),
}
