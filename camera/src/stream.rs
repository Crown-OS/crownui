use std::thread::JoinHandle;

use crossbeam_channel::{Receiver, Sender};
use media::VideoFrame;
use tokio::sync::watch;

use crate::{CameraState, capture::CaptureThread, processing::Command};

/// Decoded frames for a consumer other than the preview: a recorder, a photo, an effect.
/// Frames are dropped while it is full; drop it to stop the copies.
pub type FrameTap = Receiver<VideoFrame>;

/// A running camera: its state and taps, stopped when dropped.
pub struct CameraStream {
    state: watch::Receiver<CameraState>,
    commands: Sender<Command>,
    capture: Option<CaptureThread>,
    processing: Option<JoinHandle<()>>,
}

impl CameraStream {
    pub(crate) fn new(
        state: watch::Receiver<CameraState>,
        commands: Sender<Command>,
        capture: CaptureThread,
        processing: JoinHandle<()>,
    ) -> Self {
        Self {
            state,
            commands,
            capture: Some(capture),
            processing: Some(processing),
        }
    }

    /// Where the stream is: starting, live in a format, or failed.
    pub fn state(&self) -> watch::Receiver<CameraState> {
        self.state.clone()
    }

    /// A copy of every decoded frame from now on, holding up to `capacity` unread frames. Each
    /// frame is copied only while a tap is alive.
    pub fn tap(&self, capacity: usize) -> FrameTap {
        let (sender, tap) = crossbeam_channel::bounded(capacity.max(1));
        let _ = self.commands.send(Command::Tap(sender));
        tap
    }
}

impl Drop for CameraStream {
    fn drop(&mut self) {
        drop(self.capture.take());
        if let Some(processing) = self.processing.take() {
            let _ = processing.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send<T: Send>() {}

    #[test]
    fn a_stream_and_its_taps_can_move_to_other_threads() {
        assert_send::<CameraStream>();
        assert_send::<FrameTap>();
    }
}
