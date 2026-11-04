use thiserror::Error;

/// Where a [`MicrophoneStream`](crate::MicrophoneStream) is.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum StreamState {
    /// Connecting to PipeWire (and crown-voiced, when enhancement is on).
    #[default]
    Starting,
    /// Frames are flowing; `enhanced_by` names the crown-voiced backend, if any.
    Live {
        /// The backend and device enhancing the audio, such as `"cpu: DeepFilterNet3"`.
        enhanced_by: Option<String>,
    },
    /// Frames are flowing from the raw source because enhancement was asked for but is unavailable.
    Unenhanced(UnenhancedReason),
    /// The capture failed; the message says why.
    Failed(String),
}

/// Why a stream fell back to the raw source.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum UnenhancedReason {
    /// crown-voiced is not running or not reachable.
    #[error("crown-voiced is unavailable: {0}")]
    Unavailable(String),
    /// crown-voiced answered with an error.
    #[error("crown-voiced refused: {0}")]
    Refused(String),
    /// The connection to crown-voiced closed.
    #[error("crown-voiced disconnected")]
    Disconnected,
    /// crown-voiced did not answer in time.
    #[error("crown-voiced did not answer in time")]
    TimedOut,
}
