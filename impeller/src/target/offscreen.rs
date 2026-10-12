use graphics::GraphicsError;
use impellers::{DisplayList, ISize, PixelFormat};

use super::{
    egl::gl_proc_address,
    frame_error,
    framebuffer::Framebuffer,
    headless_egl::{HeadlessEgl, HeadlessPlatform},
};
use crate::error::InitError;

/// GLES rendering into a framebuffer object with no window; fields drop the Impeller context
/// first, then the framebuffer, then EGL. Thread-bound to the creating thread.
pub(crate) struct OffscreenTarget {
    context: impellers::Context,
    framebuffer: Framebuffer,
    egl: HeadlessEgl,
}

impl OffscreenTarget {
    pub(crate) fn new(width: u32, height: u32) -> Result<Self, InitError> {
        let egl = HeadlessEgl::new()?;
        // SAFETY: the headless context is current on this thread and outlives every GL object.
        let gl = unsafe {
            glow::Context::from_loader_function(|name| gl_proc_address(name).cast_const())
        };
        // SAFETY: the headless context is current.
        let framebuffer = unsafe { Framebuffer::new(gl, width, height) }?;
        // SAFETY: the EGL context is current on this thread and outlives the Impeller context.
        let context = unsafe { impellers::Context::new_opengl_es(gl_proc_address) }
            .map_err(InitError::Impeller)?;
        Ok(Self {
            context,
            framebuffer,
            egl,
        })
    }

    pub(crate) fn platform(&self) -> HeadlessPlatform {
        self.egl.platform()
    }

    pub(crate) fn context(&self) -> &impellers::Context {
        &self.context
    }

    pub(crate) fn activate(&self) -> Result<(), khronos_egl::Error> {
        self.egl.make_current()
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        if let Err(error) = self.framebuffer.resize(width, height) {
            tracing::warn!(%error, width, height, "resizing the offscreen framebuffer failed");
        }
    }

    pub(crate) fn presented_size(&self) -> Option<(u32, u32)> {
        Some(self.framebuffer.size())
    }

    pub(crate) fn present(&mut self, frame: &DisplayList) -> Result<bool, GraphicsError> {
        let (width, height) = self.framebuffer.size();
        // SAFETY: the framebuffer is complete, sized `width` x `height`, and outlives the surface.
        let wrapped = unsafe {
            self.context.wrap_fbo(
                self.framebuffer.id(),
                PixelFormat::RGBA8888,
                ISize::new(width.into(), height.into()),
            )
        };
        let Some(mut surface) = wrapped else {
            return Ok(false);
        };
        surface.draw_display_list(frame).map_err(frame_error)?;
        Ok(true)
    }

    /// The last frame, bottom row first, premultiplied.
    pub(crate) fn read_rgba(&self) -> (u32, u32, Vec<u8>) {
        let (width, height) = self.framebuffer.size();
        (width, height, self.framebuffer.read_rgba())
    }
}
