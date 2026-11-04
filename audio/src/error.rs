use std::io;

use thiserror::Error;

/// Why a microphone could not be opened.
#[derive(Debug, Error)]
pub enum AudioError {
    /// The PipeWire daemon could not be reached, or refused to create a stream.
    #[error("CUI0701: PipeWire is unavailable: {0}")]
    PipeWire(String),
    /// The capture thread could not be started.
    #[error("CUI0702: the capture thread could not start: {0}")]
    Thread(#[source] io::Error),
    /// The mono 48 kHz capture format could not be described to PipeWire.
    #[error("CUI0703: the capture format could not be built")]
    Format,
    /// The capture thread stopped before the stream was set up.
    #[error("CUI0704: the capture thread stopped unexpectedly")]
    Stopped,
}

impl AudioError {
    pub(crate) fn pipewire(error: pipewire::Error) -> Self {
        Self::PipeWire(error.to_string())
    }
}
