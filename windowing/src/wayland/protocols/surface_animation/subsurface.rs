use std::ptr::NonNull;

use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle, WindowHandle,
};
use sctk::reexports::client::{
    Connection, Proxy,
    protocol::{wl_subsurface::WlSubsurface, wl_surface::WlSurface},
};

use super::SurfaceAnimation;

/// A subsurface of a window with compositor-driven animation: render into it once
/// (through its raw window handle), then move, scale and fade it with
/// [`AnimatedSubsurface::animation`] without drawing another frame.
#[derive(Debug)]
pub struct AnimatedSubsurface {
    pub(super) animation: SurfaceAnimation,
    pub(super) subsurface: WlSubsurface,
    pub(super) surface: WlSurface,
    pub(super) connection: Connection,
}

impl AnimatedSubsurface {
    pub fn animation(&self) -> &SurfaceAnimation {
        &self.animation
    }

    pub fn wl_surface(&self) -> &WlSurface {
        &self.surface
    }

    /// Where the subsurface sits in its parent, in the parent's logical pixels;
    /// applied on the parent's next commit. Animated translation adds to it.
    pub fn set_position(&self, x: i32, y: i32) {
        self.subsurface.set_position(x, y);
    }

    /// Lets the subsurface's own commits apply without waiting for the parent's.
    pub fn set_desync(&self) {
        self.subsurface.set_desync();
    }
}

impl Drop for AnimatedSubsurface {
    fn drop(&mut self) {
        self.subsurface.destroy();
        self.surface.destroy();
    }
}

impl HasWindowHandle for AnimatedSubsurface {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let surface =
            NonNull::new(self.surface.id().as_ptr().cast()).ok_or(HandleError::Unavailable)?;
        let raw = RawWindowHandle::Wayland(WaylandWindowHandle::new(surface));
        // SAFETY: the wl_surface lives until this subsurface is dropped.
        Ok(unsafe { WindowHandle::borrow_raw(raw) })
    }
}

impl HasDisplayHandle for AnimatedSubsurface {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let display = NonNull::new(self.connection.display().id().as_ptr().cast())
            .ok_or(HandleError::Unavailable)?;
        let raw = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(display));
        // SAFETY: the wl_display lives as long as the held connection.
        Ok(unsafe { DisplayHandle::borrow_raw(raw) })
    }
}
