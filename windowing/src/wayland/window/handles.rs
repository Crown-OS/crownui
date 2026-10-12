use std::ptr::NonNull;

use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle, WindowHandle,
};
use sctk::{
    reexports::client::{
        Connection, Proxy,
        protocol::{wl_compositor::WlCompositor, wl_surface::WlSurface},
    },
    reexports::protocols::wp::presentation_time::client::wp_presentation::WpPresentation,
};

use super::ShellSurface;
use crate::wayland::{event_loop::WaylandPlatform, protocols::background_effects::EffectProxies};

/// A thread-safe handle to a window's surface for presenting, e.g. through a Vulkan
/// swapchain. The surface stays alive until the window is closed and every handle is gone.
#[derive(Debug, Clone)]
pub struct SurfaceHandle {
    shell: ShellSurface,
    connection: Connection,
    pub(crate) compositor: WlCompositor,
    pub(crate) effect: Option<EffectProxies>,
    pub(crate) presentation: Option<WpPresentation>,
}

impl SurfaceHandle {
    pub(crate) fn new(
        shell: ShellSurface,
        connection: Connection,
        compositor: WlCompositor,
        effect: Option<EffectProxies>,
        presentation: Option<WpPresentation>,
    ) -> Self {
        Self {
            shell,
            connection,
            compositor,
            effect,
            presentation,
        }
    }

    pub fn wl_surface(&self) -> &WlSurface {
        self.shell.wl_surface()
    }

    pub fn connection(&self) -> &Connection {
        &self.connection
    }
}

fn display_handle(connection: &Connection) -> Result<DisplayHandle<'_>, HandleError> {
    let display =
        NonNull::new(connection.display().id().as_ptr().cast()).ok_or(HandleError::Unavailable)?;
    let raw = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(display));
    // SAFETY: the wl_display lives as long as the borrowed connection.
    Ok(unsafe { DisplayHandle::borrow_raw(raw) })
}

impl HasWindowHandle for SurfaceHandle {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let surface =
            NonNull::new(self.wl_surface().id().as_ptr().cast()).ok_or(HandleError::Unavailable)?;
        let raw = RawWindowHandle::Wayland(WaylandWindowHandle::new(surface));
        // SAFETY: the held shell surface keeps the wl_surface alive for the borrow.
        Ok(unsafe { WindowHandle::borrow_raw(raw) })
    }
}

impl HasDisplayHandle for SurfaceHandle {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        display_handle(&self.connection)
    }
}

impl HasDisplayHandle for WaylandPlatform {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        display_handle(&self.connection)
    }
}
