use std::time::Duration;

use reactive::ScopeId;

use crate::Cx;

type FrameCallback = Box<dyn FnOnce(&mut Cx, Duration)>;

/// Callbacks waiting for the next animation frame, each with the scope that asked.
#[derive(Default)]
pub(crate) struct AnimationFrames {
    queued: Vec<(ScopeId, FrameCallback)>,
}

impl AnimationFrames {
    pub(crate) fn is_empty(&self) -> bool {
        self.queued.is_empty()
    }
}

impl Cx {
    /// Calls `callback` with the frame time just before the next frame is laid out, like the DOM's
    /// `requestAnimationFrame`; asking again from inside it animates every frame. For motion the
    /// render thread cannot run on its own (simulations, values that drive layout); everything
    /// else should use `.transition(..)`. Dropped if the calling component unmounts first.
    pub fn request_animation_frame(&mut self, callback: impl FnOnce(&mut Cx, Duration) + 'static) {
        let scope = self.current_scope();
        self.animation_frames
            .queued
            .push((scope, Box::new(callback)));
    }

    pub(crate) fn run_animation_frames(&mut self, now: Duration) {
        let queued = std::mem::take(&mut self.animation_frames.queued);
        for (scope, callback) in queued {
            if self.is_scope_alive(scope) {
                let previous = self.set_current_scope(scope);
                callback(self, now);
                self.set_current_scope(previous);
            }
        }
        self.apply_pending();
    }
}
