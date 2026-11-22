use crossbeam_channel::{Receiver, Sender, TrySendError};
use graphics::{ImageId, StreamFrame};

use crate::Cx;

/// Frames in flight per window before producers see [`PushError::Full`]. The render thread keeps
/// only the latest frame of each stream, so this only absorbs bursts while it is busy.
const FRAMES_IN_FLIGHT: usize = 4;

/// A frame on its way to the render thread.
#[derive(Debug)]
pub struct StreamedFrame {
    /// The image it replaces.
    pub image: ImageId,
    /// The new pixels.
    pub frame: StreamFrame,
}

/// Why [`ImageStream::push`] dropped a frame.
#[derive(Debug, thiserror::Error)]
pub enum PushError {
    /// The render thread is behind; the frame was dropped and the next one may go through.
    #[error("the render thread is behind")]
    Full,
    /// The window is gone; stop producing.
    #[error("the window is closed")]
    Closed,
}

/// The producer end of a live image made by [`Cx::image_stream`]: `Send`, cheap to clone, and
/// drawn like any registered image by [`ImageStream::image`]. Each pushed frame goes straight to
/// the render thread, which redraws only what shows the image, without waking the UI thread.
#[derive(Debug, Clone)]
pub struct ImageStream {
    image: ImageId,
    frames: Sender<StreamedFrame>,
}

impl ImageStream {
    /// The id to draw the stream by.
    pub fn image(&self) -> ImageId {
        self.image
    }

    /// Shows `frame` from the next display refresh on.
    pub fn push(&self, frame: StreamFrame) -> Result<(), PushError> {
        let streamed = StreamedFrame {
            image: self.image,
            frame,
        };
        self.frames.try_send(streamed).map_err(|error| match error {
            TrySendError::Full(_) => PushError::Full,
            TrySendError::Disconnected(_) => PushError::Closed,
        })
    }
}

/// One window's stream channel: every [`ImageStream`] sends into it, the render thread drains it.
#[derive(Debug)]
pub(crate) struct StreamChannel {
    sender: Sender<StreamedFrame>,
    receiver: Receiver<StreamedFrame>,
}

impl Default for StreamChannel {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::bounded(FRAMES_IN_FLIGHT);
        Self { sender, receiver }
    }
}

impl StreamChannel {
    pub(crate) fn receiver(&self) -> Receiver<StreamedFrame> {
        self.receiver.clone()
    }
}

impl Cx {
    /// A live image, black until its first frame, released when the current scope ends. Move the
    /// returned stream to the thread that produces frames (a camera, a video decoder).
    pub fn image_stream(&mut self) -> ImageStream {
        const OPAQUE_BLACK: [u8; 4] = [0, 0, 0, 255];
        let image = self.register_image(1, 1, OPAQUE_BLACK.to_vec());
        self.release_image_on_cleanup(image);
        ImageStream {
            image,
            frames: self.streams.sender.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame() -> StreamFrame {
        StreamFrame::new(1, 1, vec![9, 9, 9, 255]).expect("one pixel")
    }

    #[test]
    fn frames_reach_the_receiver_until_the_window_falls_behind() {
        let channel = StreamChannel::default();
        let stream = ImageStream {
            image: ImageId(3),
            frames: channel.sender.clone(),
        };
        for _ in 0..FRAMES_IN_FLIGHT {
            assert!(stream.push(frame()).is_ok());
        }
        assert!(matches!(stream.push(frame()), Err(PushError::Full)));
        let received: Vec<ImageId> = channel.receiver().try_iter().map(|f| f.image).collect();
        assert_eq!(received, [ImageId(3); FRAMES_IN_FLIGHT]);
    }

    #[test]
    fn a_closed_window_stops_the_producer() {
        let channel = StreamChannel::default();
        let stream = ImageStream {
            image: ImageId(1),
            frames: channel.sender.clone(),
        };
        drop(channel);
        assert!(matches!(stream.push(frame()), Err(PushError::Closed)));
    }
}
