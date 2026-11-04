use std::{ops::ControlFlow, time::Duration};

use crossbeam_channel::{Receiver, TryRecvError};
use graphics::Renderer;
use windowing::SurfaceState;

use crate::{
    render::RenderTree,
    render_thread::{RenderMessage, merge_surface_state},
};

/// What the render thread has taken in from the UI thread since its last frame.
#[derive(Debug, Default)]
pub(crate) struct RenderInbox {
    pub(crate) pending_surface: Option<SurfaceState>,
    pub(crate) dirty: bool,
    pub(crate) has_viewport: bool,
    pub(crate) paused: bool,
    pub(crate) ui_waits_for_frames: bool,
}

impl RenderInbox {
    /// Whether a frame should be drawn now that `tree` is or is not animating.
    pub(crate) fn wants_frame(&self, tree: &RenderTree) -> bool {
        !self.paused && (self.dirty || tree.is_animating())
    }

    /// Applies every message already sent; breaks when the UI thread is gone or shut it down.
    pub(crate) fn drain<R: Renderer>(
        &mut self,
        receiver: &Receiver<RenderMessage<R::TextLayout>>,
        tree: &mut RenderTree,
        renderer: &mut R,
        now: Duration,
    ) -> ControlFlow<()> {
        loop {
            match receiver.try_recv() {
                Ok(message) => self.receive(message, tree, renderer, now)?,
                Err(TryRecvError::Empty) => return ControlFlow::Continue(()),
                Err(TryRecvError::Disconnected) => return ControlFlow::Break(()),
            }
        }
    }

    pub(crate) fn receive<R: Renderer>(
        &mut self,
        message: RenderMessage<R::TextLayout>,
        tree: &mut RenderTree,
        renderer: &mut R,
        now: Duration,
    ) -> ControlFlow<()> {
        match message {
            RenderMessage::Shutdown => return ControlFlow::Break(()),
            RenderMessage::Paused(paused) => {
                tracing::debug!(paused, "render paused");
                self.paused = paused;
                self.dirty = true;
            }
            RenderMessage::AnimationFrames(waiting) => self.ui_waits_for_frames = waiting,
            RenderMessage::Viewport {
                width,
                height,
                scale,
            } => {
                tracing::debug!(width, height, scale, "render viewport");
                renderer.set_viewport(width, height, scale);
                self.has_viewport = true;
                self.dirty = true;
            }
            RenderMessage::Batch(batch) => {
                let (ops, surface_state, releases) = batch.upload(renderer);
                if let Some(state) = surface_state {
                    merge_surface_state(&mut self.pending_surface, state);
                }
                tree.apply(ops, now, renderer);
                tree.release_when_unused(releases);
                self.dirty = true;
            }
        }
        ControlFlow::Continue(())
    }
}
