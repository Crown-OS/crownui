use crate::{FrameTiming, PlatformSurface, SurfaceError, SurfaceState, WindowId};

/// A surface whose frames are ready immediately; it remembers the last committed state.
#[derive(Debug, Clone)]
pub struct HeadlessSurface {
    window: WindowId,
    presented_frames: u64,
    last_state: SurfaceState,
}

impl HeadlessSurface {
    pub(crate) fn new(window: WindowId) -> Self {
        Self {
            window,
            presented_frames: 0,
            last_state: SurfaceState::default(),
        }
    }

    /// The window this surface presents.
    pub fn window(&self) -> WindowId {
        self.window
    }

    /// How many frames were presented.
    pub fn presented_frames(&self) -> u64 {
        self.presented_frames
    }

    /// The state committed with the latest frame.
    pub fn last_state(&self) -> &SurfaceState {
        &self.last_state
    }
}

impl PlatformSurface for HeadlessSurface {
    fn pre_present_notify(&mut self) {
        self.presented_frames += 1;
    }

    fn wait_for_frame(&mut self) -> Result<FrameTiming, SurfaceError> {
        Ok(FrameTiming::default())
    }

    fn apply(&mut self, state: &SurfaceState) {
        self.last_state.clone_from(state);
    }
}
