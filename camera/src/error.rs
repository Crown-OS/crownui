use std::io;

use thiserror::Error;

/// Why a camera could not be opened.
#[derive(Debug, Error)]
pub enum CameraError {
    /// The PipeWire daemon could not be reached, or refused to create a stream.
    #[error("CUI0802: PipeWire is unavailable: {0}")]
    PipeWire(String),
    /// A capture or decoding thread could not be started.
    #[error("CUI0803: a camera thread could not start: {0}")]
    Thread(#[source] io::Error),
    /// The formats to offer the camera could not be described to PipeWire.
    #[error("CUI0804: the camera formats could not be built")]
    Format,
    /// The capture thread stopped before the stream was set up.
    #[error("CUI0805: the capture thread stopped unexpectedly")]
    Stopped,
}

impl CameraError {
    pub(crate) fn pipewire(error: pipewire::Error) -> Self {
        Self::PipeWire(error.to_string())
    }
}
