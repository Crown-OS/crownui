use std::path::PathBuf;

use thiserror::Error;

/// Why a capture failed.
#[derive(Debug, Error)]
pub enum CaptureError {
    /// The offscreen renderer or a frame failed.
    #[error("rendering failed: {0}")]
    Graphics(#[from] graphics::GraphicsError),
    /// The scene could not be serialized.
    #[error("serializing the scene failed: {0}")]
    Json(#[from] serde_json::Error),
    /// Writing a capture file failed.
    #[error("writing {path} failed: {source}")]
    Write {
        /// The file being written.
        path: PathBuf,
        /// The I/O error.
        source: std::io::Error,
    },
}
