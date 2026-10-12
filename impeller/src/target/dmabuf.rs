use std::{collections::HashMap, ffi::c_void, path::Path};

use glow::{HasContext, NativeFramebuffer, NativeRenderbuffer};
use graphics::GraphicsError;
use impellers::{DisplayList, ISize, PixelFormat};
use khronos_egl as egl;

use super::{
    egl::gl_proc_address, frame_error, framebuffer::Framebuffer, headless_egl::HeadlessEgl,
};
use crate::{dmabuf::DmabufImage, error::InitError};

const EGL_LINUX_DMA_BUF_EXT: egl::Enum = 0x3270;
const EGL_LINUX_DRM_FOURCC_EXT: egl::Attrib = 0x3271;
const EGL_DMA_BUF_PLANE0_FD_EXT: egl::Attrib = 0x3272;
const EGL_DMA_BUF_PLANE0_OFFSET_EXT: egl::Attrib = 0x3273;
const EGL_DMA_BUF_PLANE0_PITCH_EXT: egl::Attrib = 0x3274;
const EGL_DMA_BUF_PLANE0_MODIFIER_LO_EXT: egl::Attrib = 0x3443;
const EGL_DMA_BUF_PLANE0_MODIFIER_HI_EXT: egl::Attrib = 0x3444;
/// `DRM_FORMAT_MOD_INVALID`: the allocator chose the layout implicitly.
const MODIFIER_INVALID: u64 = 0x00ff_ffff_ffff_ffff;

type ImageTargetRenderbufferStorage = unsafe extern "system" fn(u32, *const c_void);

/// One imported dma-buf, as a renderbuffer the multisampled frame resolves into.
struct Import {
    image: egl::Image,
    renderbuffer: NativeRenderbuffer,
    framebuffer: NativeFramebuffer,
}

/// GLES rendering into dma-bufs the host allocated and composites itself; Impeller draws into
/// a multisampled framebuffer and each frame is resolved into the selected buffer. Fields drop
/// the Impeller context first and EGL last; thread-bound to the creating thread.
pub(crate) struct DmabufTarget {
    context: impellers::Context,
    framebuffer: Framebuffer,
    imports: HashMap<u64, Import>,
    selected: Option<u64>,
    image_target: ImageTargetRenderbufferStorage,
    egl: HeadlessEgl,
}

impl DmabufTarget {
    pub(crate) fn new(render_node: &Path, width: u32, height: u32) -> Result<Self, InitError> {
        let egl = HeadlessEgl::on_render_node(render_node)?;
        let image_target = gl_proc_address("glEGLImageTargetRenderbufferStorageOES");
        if image_target.is_null() {
            return Err(InitError::MissingGlFunction(
                "glEGLImageTargetRenderbufferStorageOES",
            ));
        }
        // SAFETY: the name resolves to this signature per OES_EGL_image.
        let image_target: ImageTargetRenderbufferStorage =
            unsafe { std::mem::transmute(image_target) };
        // SAFETY: the context is current on this thread and outlives every GL object.
        let gl = unsafe {
            glow::Context::from_loader_function(|name| gl_proc_address(name).cast_const())
        };
        // SAFETY: the context is current.
        let framebuffer = unsafe { Framebuffer::new(gl, width, height) }?;
        // SAFETY: the EGL context is current on this thread and outlives the Impeller context.
        let context = unsafe { impellers::Context::new_opengl_es(gl_proc_address) }
            .map_err(InitError::Impeller)?;
        Ok(Self {
            context,
            framebuffer,
            imports: HashMap::new(),
            selected: None,
            image_target,
            egl,
        })
    }

    pub(crate) fn context(&self) -> &impellers::Context {
        &self.context
    }

    pub(crate) fn activate(&self) -> Result<(), egl::Error> {
        self.egl.make_current()
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        if let Err(error) = self.framebuffer.resize(width, height) {
            tracing::warn!(%error, width, height, "resizing the dma-buf framebuffer failed");
        }
    }

    pub(crate) fn presented_size(&self) -> Option<(u32, u32)> {
        Some(self.framebuffer.size())
    }

    pub(crate) fn import(
        &mut self,
        key: u64,
        image: &DmabufImage<'_>,
    ) -> Result<(), GraphicsError> {
        self.forget(key);
        let modifier = if image.modifier == MODIFIER_INVALID {
            None
        } else {
            Some(image.modifier)
        };
        let mut attributes = vec![
            egl::WIDTH as egl::Attrib,
            image.width as egl::Attrib,
            egl::HEIGHT as egl::Attrib,
            image.height as egl::Attrib,
            EGL_LINUX_DRM_FOURCC_EXT,
            image.fourcc as egl::Attrib,
            EGL_DMA_BUF_PLANE0_FD_EXT,
            std::os::fd::AsRawFd::as_raw_fd(&image.plane.fd) as egl::Attrib,
            EGL_DMA_BUF_PLANE0_OFFSET_EXT,
            image.plane.offset as egl::Attrib,
            EGL_DMA_BUF_PLANE0_PITCH_EXT,
            image.plane.stride as egl::Attrib,
        ];
        if let Some(modifier) = modifier {
            attributes.extend([
                EGL_DMA_BUF_PLANE0_MODIFIER_LO_EXT,
                (modifier & 0xffff_ffff) as egl::Attrib,
                EGL_DMA_BUF_PLANE0_MODIFIER_HI_EXT,
                (modifier >> 32) as egl::Attrib,
            ]);
        }
        attributes.push(egl::ATTRIB_NONE);
        let api = &egl::API;
        // SAFETY: a dma-buf import takes no context and no client buffer; EGL dups the fd.
        let image_handle = unsafe {
            api.create_image(
                self.egl.display(),
                egl::Context::from_ptr(egl::NO_CONTEXT),
                EGL_LINUX_DMA_BUF_EXT,
                egl::ClientBuffer::from_ptr(std::ptr::null_mut()),
                &attributes,
            )
        }
        .map_err(frame_error)?;

        let gl = self.framebuffer.gl();
        // SAFETY: the context is current; the image outlives the renderbuffer it backs.
        let created = unsafe {
            let renderbuffer = gl.create_renderbuffer().map_err(frame_error)?;
            gl.bind_renderbuffer(glow::RENDERBUFFER, Some(renderbuffer));
            (self.image_target)(glow::RENDERBUFFER, image_handle.as_ptr());
            let framebuffer = gl.create_framebuffer().map_err(frame_error)?;
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));
            gl.framebuffer_renderbuffer(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::RENDERBUFFER,
                Some(renderbuffer),
            );
            let status = gl.check_framebuffer_status(glow::FRAMEBUFFER);
            (renderbuffer, framebuffer, status)
        };
        let (renderbuffer, framebuffer, status) = created;
        let import = Import {
            image: image_handle,
            renderbuffer,
            framebuffer,
        };
        if status != glow::FRAMEBUFFER_COMPLETE {
            self.release(import);
            return Err(frame_error(format!(
                "dma-buf framebuffer incomplete (status {status:#x})"
            )));
        }
        self.imports.insert(key, import);
        Ok(())
    }

    pub(crate) fn forget(&mut self, key: u64) {
        if self.selected == Some(key) {
            self.selected = None;
        }
        if let Some(import) = self.imports.remove(&key) {
            self.release(import);
        }
    }

    pub(crate) fn select(&mut self, key: u64) -> bool {
        let known = self.imports.contains_key(&key);
        self.selected = known.then_some(key);
        known
    }

    fn release(&self, import: Import) {
        let gl = self.framebuffer.gl();
        // SAFETY: the context is current and these objects came from it.
        unsafe {
            gl.delete_framebuffer(import.framebuffer);
            gl.delete_renderbuffer(import.renderbuffer);
        }
        if let Err(error) = egl::API.destroy_image(self.egl.display(), import.image) {
            tracing::warn!(%error, "destroying an imported dma-buf failed");
        }
    }

    /// Draws `frame`, resolves it into the selected buffer and waits for the GPU, so the host
    /// may sample the buffer as soon as this returns.
    pub(crate) fn present(&mut self, frame: &DisplayList) -> Result<bool, GraphicsError> {
        let Some(target) = self
            .selected
            .and_then(|key| self.imports.get(&key))
            .map(|import| import.framebuffer)
        else {
            return Ok(false);
        };
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
        drop(surface);
        self.framebuffer.resolve_into(target);
        // SAFETY: the context is current.
        unsafe { self.framebuffer.gl().finish() };
        Ok(true)
    }

    /// The last frame, bottom row first, premultiplied; for tests.
    pub(crate) fn read_rgba(&self) -> (u32, u32, Vec<u8>) {
        let (width, height) = self.framebuffer.size();
        (width, height, self.framebuffer.read_rgba())
    }
}

impl Drop for DmabufTarget {
    fn drop(&mut self) {
        let _ = self.activate();
        for (_, import) in std::mem::take(&mut self.imports) {
            self.release(import);
        }
    }
}
