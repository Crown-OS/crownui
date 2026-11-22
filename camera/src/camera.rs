use media::VideoFrame;
use tokio::sync::watch;

use crate::{
    CameraError, CameraOptions, CameraState, CameraStream,
    capture::CaptureThread,
    processing::{self, ProcessingLinks},
};

/// Encoded frames waiting for the decoder. More only adds latency: when decoding falls behind,
/// the newest frames are dropped.
const ENCODED_IN_FLIGHT: usize = 2;

/// Opens cameras.
#[derive(Debug, Clone, Copy)]
pub struct Camera;

impl Camera {
    /// Starts capturing as `options` describe and returns once PipeWire accepted the stream.
    /// Every decoded frame is handed to `preview` on the decoding thread, after the taps.
    ///
    /// # Errors
    ///
    /// [`CameraError`] when PipeWire is unreachable or a camera thread cannot start.
    pub fn open(
        options: CameraOptions,
        preview: impl FnMut(VideoFrame) + Send + 'static,
    ) -> Result<CameraStream, CameraError> {
        let (event_sender, events) = crossbeam_channel::bounded(ENCODED_IN_FLIGHT);
        let (recycle, recycled) = crossbeam_channel::bounded(ENCODED_IN_FLIGHT + 1);
        let (command_sender, commands) = crossbeam_channel::unbounded();
        let (state_sender, state) = watch::channel(CameraState::Starting);
        let links = ProcessingLinks {
            events,
            recycle,
            commands,
            state: state_sender,
        };
        let processing = processing::spawn(links, preview)?;
        let capture = CaptureThread::spawn(options, event_sender, recycled)?;
        Ok(CameraStream::new(
            state,
            command_sender,
            capture,
            processing,
        ))
    }
}
