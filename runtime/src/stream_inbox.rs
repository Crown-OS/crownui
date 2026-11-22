use std::collections::HashMap;

use crossbeam_channel::Receiver;
use graphics::{ImageId, Renderer, StreamFrame};
use view::StreamedFrame;

use crate::render::RenderTree;

/// The latest frame of each live image since the last drawn frame. Older frames are dropped
/// unseen, so a paused or slow window never uploads more than one frame per stream.
#[derive(Debug, Default)]
pub(crate) struct StreamInbox {
    latest: HashMap<ImageId, StreamFrame>,
}

impl StreamInbox {
    pub(crate) fn stash(&mut self, streamed: StreamedFrame) {
        self.latest.insert(streamed.image, streamed.frame);
    }

    /// Stashes every frame already sent; true while a frame waits to be drawn.
    pub(crate) fn take(&mut self, frames: &Receiver<StreamedFrame>) -> bool {
        for streamed in frames.try_iter() {
            self.stash(streamed);
        }
        !self.latest.is_empty()
    }

    /// Uploads every stashed frame and marks what draws it for re-recording. Frames of images the
    /// renderer no longer holds (their stream's component unmounted) are dropped.
    pub(crate) fn apply<R: Renderer>(&mut self, tree: &mut RenderTree, renderer: &mut R) {
        for (image, frame) in self.latest.drain() {
            if renderer.replace_image(image, frame).is_ok() {
                tree.image_changed(image);
            }
        }
    }
}
