use crate::CaptureFormat;

/// Where a [`CameraStream`](crate::CameraStream) is.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum CameraState {
    /// Connecting to PipeWire and negotiating a format.
    #[default]
    Starting,
    /// Frames are flowing in `format`.
    Live(CaptureFormat),
    /// The camera failed or went away; the message says why.
    Failed(String),
}
