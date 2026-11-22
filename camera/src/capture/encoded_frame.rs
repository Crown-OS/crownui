use media::ClockNanos;

use crate::CaptureFormat;

/// One frame as the camera sent it: compressed or packed YUV.
#[derive(Debug)]
pub(crate) struct EncodedFrame {
    pub(crate) format: CaptureFormat,
    pub(crate) stride: u32,
    pub(crate) pts: ClockNanos,
    pub(crate) bytes: Vec<u8>,
}

/// What the capture thread tells the decoding thread.
#[derive(Debug)]
pub(crate) enum CaptureEvent {
    Frame(EncodedFrame),
    Failed(String),
}
