use tokio::sync::watch;

use crate::{
    Enhancement, FrameConsumer, Level, StreamState,
    capture::{CaptureThread, Command},
};

/// A running capture: frames, level and state, stopped when dropped.
pub struct MicrophoneStream {
    frames: Option<FrameConsumer>,
    level: watch::Receiver<Level>,
    state: watch::Receiver<StreamState>,
    capture: CaptureThread,
}

impl MicrophoneStream {
    pub(crate) fn new(
        frames: FrameConsumer,
        level: watch::Receiver<Level>,
        state: watch::Receiver<StreamState>,
        capture: CaptureThread,
    ) -> Self {
        Self {
            frames: Some(frames),
            level,
            state,
            capture,
        }
    }

    /// The frame ring, once: move it to the worker that consumes the audio. Frames are dropped
    /// while nobody pops them and the ring is full.
    pub fn take_frames(&mut self) -> Option<FrameConsumer> {
        self.frames.take()
    }

    /// The level of every captured frame.
    pub fn level(&self) -> watch::Receiver<Level> {
        self.level.clone()
    }

    /// Where the stream is: starting, live (and how it is enhanced), or failed.
    pub fn state(&self) -> watch::Receiver<StreamState> {
        self.state.clone()
    }

    /// Switches enhancement without stopping the stream; the outcome shows in [`Self::state`].
    pub fn set_enhancement(&self, enhancement: Enhancement) {
        self.capture.send(Command::SetEnhancement(enhancement));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send<T: Send>() {}

    #[test]
    fn a_stream_can_move_to_another_thread() {
        assert_send::<MicrophoneStream>();
        assert_send::<FrameConsumer>();
    }
}
