use std::ffi::c_void;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};

use crate::error::InitError;

/// The `wl_display` and `wl_surface` a renderer presents to.
#[derive(Debug, Clone, Copy)]
pub(crate) struct WaylandSurfacePointers {
    pub(crate) display: *mut c_void,
    pub(crate) surface: *mut c_void,
}

impl WaylandSurfacePointers {
    pub(crate) fn from_window<S: HasWindowHandle + HasDisplayHandle>(
        window: &S,
    ) -> Result<Self, InitError> {
        let surface = match window.window_handle().map_err(InitError::Handle)?.as_raw() {
            RawWindowHandle::Wayland(handle) => handle.surface.as_ptr(),
            other => return Err(InitError::UnsupportedWindow(format!("{other:?}"))),
        };
        let display = match window.display_handle().map_err(InitError::Handle)?.as_raw() {
            RawDisplayHandle::Wayland(handle) => handle.display.as_ptr(),
            other => return Err(InitError::UnsupportedDisplay(format!("{other:?}"))),
        };
        Ok(Self { display, surface })
    }
}
