use std::{os::fd::BorrowedFd, path::Path};

use graphics::{CreateRenderer, GraphicsError, Renderer};

use crate::{ImpellerBackend, ImpellerRenderer};

/// A windowless render target on one GPU, drawing into dma-bufs the host allocates there and
/// composites itself; see [`ImpellerRenderer::import_dmabuf`].
#[derive(Debug, Clone, Copy)]
pub struct RenderNode<'a> {
    /// The DRM render node of the GPU the buffers live on, e.g. `/dev/dri/renderD128`.
    pub path: &'a Path,
    /// Width in device pixels.
    pub width: u32,
    /// Height in device pixels.
    pub height: u32,
    /// Logical-to-device scale.
    pub scale: f32,
}

/// The single plane of an RGBA dma-buf.
#[derive(Debug, Clone, Copy)]
pub struct DmabufPlane<'a> {
    /// The buffer's file descriptor; EGL takes its own reference on import.
    pub fd: BorrowedFd<'a>,
    /// Byte offset of the first pixel.
    pub offset: u32,
    /// Bytes per row.
    pub stride: u32,
}

/// A dma-buf to render into: single-plane, `DRM_FORMAT_ABGR8888` or `XBGR8888`.
#[derive(Debug, Clone, Copy)]
pub struct DmabufImage<'a> {
    /// Width in pixels; must match the renderer's viewport.
    pub width: u32,
    /// Height in pixels; must match the renderer's viewport.
    pub height: u32,
    /// The DRM fourcc code.
    pub fourcc: u32,
    /// The DRM format modifier, or `DRM_FORMAT_MOD_INVALID` for an implicit layout.
    pub modifier: u64,
    /// The pixels.
    pub plane: DmabufPlane<'a>,
}

impl CreateRenderer<RenderNode<'_>> for ImpellerBackend {
    fn create_renderer(target: &RenderNode<'_>) -> Result<ImpellerRenderer, GraphicsError> {
        let mut renderer =
            ImpellerRenderer::on_render_node(target.path, target.width, target.height)?;
        renderer.set_viewport(target.width, target.height, target.scale);
        Ok(renderer)
    }
}
