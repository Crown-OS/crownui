use graphics::GraphicsError;
use impellers::{DisplayList, ISize, PixelFormat};
use wayland_egl::WlEglSurface;

use super::{
    egl::{EglWindow, gl_proc_address},
    frame_error,
    wayland::WaylandSurfacePointers,
};
use crate::error::InitError;

const DEFAULT_FRAMEBUFFER: u64 = 0;

/// GLES through EGL on a `wl_egl_window`; fields drop the Impeller context first, then EGL,
/// then the native window. Thread-bound to the creating thread.
pub(crate) struct GlesTarget {
    context: impellers::Context,
    egl: EglWindow,
    native_window: WlEglSurface,
    size: (u32, u32),
}

impl GlesTarget {
    pub(crate) fn new(window: WaylandSurfacePointers) -> Result<Self, InitError> {
        // SAFETY: the caller keeps the wl_surface alive longer than the renderer.
        let native_window = unsafe { WlEglSurface::new_from_raw(window.surface.cast(), 1, 1) }?;
        // SAFETY: the caller keeps the wl_display alive; the native window is dropped after EGL.
        let egl = unsafe { EglWindow::new(window.display, &native_window) }?;
        // SAFETY: the EGL context is current on this thread and outlives the Impeller context.
        let context = unsafe { impellers::Context::new_opengl_es(gl_proc_address) }
            .map_err(InitError::Impeller)?;
        Ok(Self {
            context,
            egl,
            native_window,
            size: (1, 1),
        })
    }

    pub(crate) fn context(&self) -> &impellers::Context {
        &self.context
    }

    pub(crate) fn activate(&self) -> Result<(), khronos_egl::Error> {
        self.egl.make_current()
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        let to_egl = |pixels: u32| i32::try_from(pixels).unwrap_or(i32::MAX);
        self.native_window
            .resize(to_egl(width), to_egl(height), 0, 0);
        self.size = (width, height);
    }

    pub(crate) fn presented_size(&self) -> Option<(u32, u32)> {
        Some(self.size)
    }

    pub(crate) fn present(&mut self, frame: &DisplayList) -> Result<bool, GraphicsError> {
        let (width, height) = self.size;
        // SAFETY: the default framebuffer of the current EGL surface is complete and sized by
        // the last resize; the wrapped surface is dropped before swapping.
        let wrapped = unsafe {
            self.context.wrap_fbo(
                DEFAULT_FRAMEBUFFER,
                PixelFormat::RGBA8888,
                ISize::new(width.into(), height.into()),
            )
        };
        let Some(mut surface) = wrapped else {
            return Ok(false);
        };
        surface.draw_display_list(frame).map_err(frame_error)?;
        drop(surface);
        self.egl.swap_buffers().map_err(frame_error)?;
        Ok(true)
    }
}
