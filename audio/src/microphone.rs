use rtrb::RingBuffer;
use tokio::sync::watch;

use crate::{
    AudioError, Level, MicrophoneOptions, MicrophoneStream, StreamState,
    capture::{CaptureThread, FrameSink},
};

/// One second of frames: how far a worker may fall behind before frames are dropped.
const RING_FRAMES: usize = 100;

/// Opens microphones.
#[derive(Debug, Clone, Copy)]
pub struct Microphone;

impl Microphone {
    /// Starts capturing as `options` describe and returns once PipeWire accepted the stream.
    ///
    /// Enhancement is negotiated with crown-voiced in the background; follow
    /// [`MicrophoneStream::state`] for the outcome.
    ///
    /// # Errors
    ///
    /// [`AudioError`] when PipeWire is unreachable or the capture thread cannot start.
    pub fn open(options: MicrophoneOptions) -> Result<MicrophoneStream, AudioError> {
        let (producer, frames) = RingBuffer::new(RING_FRAMES);
        let (level_sender, level) = watch::channel(Level::default());
        let (state_sender, state) = watch::channel(StreamState::Starting);
        let capture = CaptureThread::spawn(
            options,
            FrameSink::new(producer, level_sender),
            state_sender,
        )?;
        Ok(MicrophoneStream::new(frames, level, state, capture))
    }
}
