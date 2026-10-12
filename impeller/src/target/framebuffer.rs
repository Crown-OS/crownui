use glow::{HasContext, NativeFramebuffer, NativeRenderbuffer};

use super::MSAA_SAMPLES;
use crate::error::InitError;

/// A multisampled RGBA8 + stencil8 framebuffer object Impeller renders into, plus a single-sample
/// RGBA8 framebuffer it resolves into for reading; GL objects are deleted on drop, so the owning
/// GL context must be current then.
pub(crate) struct Framebuffer {
    gl: Box<glow::Context>,
    framebuffer: NativeFramebuffer,
    color: NativeRenderbuffer,
    stencil: NativeRenderbuffer,
    resolve_framebuffer: NativeFramebuffer,
    resolve_color: NativeRenderbuffer,
    samples: i32,
    width: u32,
    height: u32,
}

impl Framebuffer {
    /// # Safety
    /// The GL context `gl` loads from must be current on this thread.
    pub(crate) unsafe fn new(
        gl: glow::Context,
        width: u32,
        height: u32,
    ) -> Result<Self, InitError> {
        // SAFETY: the caller guarantees a current context.
        let (framebuffer, color, stencil, resolve_framebuffer, resolve_color, max_samples) = unsafe {
            (
                gl.create_framebuffer().map_err(InitError::Gl)?,
                gl.create_renderbuffer().map_err(InitError::Gl)?,
                gl.create_renderbuffer().map_err(InitError::Gl)?,
                gl.create_framebuffer().map_err(InitError::Gl)?,
                gl.create_renderbuffer().map_err(InitError::Gl)?,
                gl.get_parameter_i32(glow::MAX_SAMPLES),
            )
        };
        let mut created = Self {
            gl: Box::new(gl),
            framebuffer,
            color,
            stencil,
            resolve_framebuffer,
            resolve_color,
            samples: MSAA_SAMPLES.min(max_samples).max(0),
            width: 0,
            height: 0,
        };
        created.resize(width, height)?;
        Ok(created)
    }

    /// Reallocates storage at `width` x `height` (each at least 1).
    pub(crate) fn resize(&mut self, width: u32, height: u32) -> Result<(), InitError> {
        let (width, height) = (width.max(1), height.max(1));
        let to_gl = |pixels: u32| i32::try_from(pixels).unwrap_or(i32::MAX);
        let gl = &self.gl;
        let attachments = [
            (
                self.framebuffer,
                self.color,
                glow::RGBA8,
                glow::COLOR_ATTACHMENT0,
                self.samples,
            ),
            (
                self.framebuffer,
                self.stencil,
                glow::STENCIL_INDEX8,
                glow::STENCIL_ATTACHMENT,
                self.samples,
            ),
            (
                self.resolve_framebuffer,
                self.resolve_color,
                glow::RGBA8,
                glow::COLOR_ATTACHMENT0,
                0,
            ),
        ];
        // SAFETY: the context that created these objects is current (type invariant).
        let incomplete = unsafe {
            for (framebuffer, renderbuffer, format, attachment, samples) in attachments {
                gl.bind_renderbuffer(glow::RENDERBUFFER, Some(renderbuffer));
                gl.renderbuffer_storage_multisample(
                    glow::RENDERBUFFER,
                    samples,
                    format,
                    to_gl(width),
                    to_gl(height),
                );
                gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));
                gl.framebuffer_renderbuffer(
                    glow::FRAMEBUFFER,
                    attachment,
                    glow::RENDERBUFFER,
                    Some(renderbuffer),
                );
            }
            [self.framebuffer, self.resolve_framebuffer]
                .into_iter()
                .map(|framebuffer| {
                    gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));
                    gl.check_framebuffer_status(glow::FRAMEBUFFER)
                })
                .find(|&status| status != glow::FRAMEBUFFER_COMPLETE)
        };
        if let Some(status) = incomplete {
            return Err(InitError::IncompleteFramebuffer(status));
        }
        (self.width, self.height) = (width, height);
        Ok(())
    }

    pub(crate) fn gl(&self) -> &glow::Context {
        &self.gl
    }

    /// Resolves the multisampled frame into `target`, which must be complete at this size.
    pub(crate) fn resolve_into(&self, target: NativeFramebuffer) {
        let (width, height) = (self.width as i32, self.height as i32);
        let gl = &self.gl;
        // SAFETY: the context is current and both framebuffers are complete at width x height.
        unsafe {
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(self.framebuffer));
            gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(target));
            gl.blit_framebuffer(
                0,
                0,
                width,
                height,
                0,
                0,
                width,
                height,
                glow::COLOR_BUFFER_BIT,
                glow::NEAREST,
            );
        }
    }

    pub(crate) fn id(&self) -> u64 {
        u64::from(self.framebuffer.0.get())
    }

    pub(crate) fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// The framebuffer's resolved RGBA8 contents, bottom row first, as GL stores them.
    pub(crate) fn read_rgba(&self) -> Vec<u8> {
        let mut rgba = vec![0; self.width as usize * self.height as usize * 4];
        let (width, height) = (self.width as i32, self.height as i32);
        let gl = &self.gl;
        // SAFETY: the context is current, both framebuffers are complete at width x height, and
        // `rgba` holds exactly width * height RGBA8 pixels.
        unsafe {
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(self.framebuffer));
            gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(self.resolve_framebuffer));
            gl.blit_framebuffer(
                0,
                0,
                width,
                height,
                0,
                0,
                width,
                height,
                glow::COLOR_BUFFER_BIT,
                glow::NEAREST,
            );
            gl.bind_buffer(glow::PIXEL_PACK_BUFFER, None);
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(self.resolve_framebuffer));
            gl.pixel_store_i32(glow::PACK_ALIGNMENT, 1);
            gl.read_pixels(
                0,
                0,
                width,
                height,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelPackData::Slice(Some(&mut rgba)),
            );
        }
        rgba
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        let gl = &self.gl;
        // SAFETY: the owner keeps the creating context current while dropping.
        unsafe {
            gl.delete_framebuffer(self.framebuffer);
            gl.delete_renderbuffer(self.color);
            gl.delete_renderbuffer(self.stencil);
            gl.delete_framebuffer(self.resolve_framebuffer);
            gl.delete_renderbuffer(self.resolve_color);
        }
    }
}
