use std::{borrow::Cow, collections::HashMap};

use geometry::CornerShape;
use graphics::{BoundaryId, Composition, DrawList, GraphicsError, ImageId, Renderer, TextLayoutId};
use impellers::DisplayList;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::{
    composite::{Viewport, compose_frame},
    dmabuf::DmabufImage,
    error::InitError,
    pixels::Pixels,
    resources::{DrawResources, UploadedTexture},
    target::{
        DmabufTarget, OffscreenTarget, RenderApi, RenderTarget, WaylandSurfacePointers, frame_error,
    },
    text::ImpellerTextLayout,
    translate::translate_draw_list,
    util::pixels::premultiplied_rgba,
};

/// Draws compositions with Impeller on one Wayland surface. Lives on the render thread.
pub struct ImpellerRenderer {
    resources: DrawResources,
    boundaries: HashMap<BoundaryId, DisplayList>,
    target: RenderTarget,
    viewport: Viewport,
    corners: CornerShape,
    frames_presented: u64,
}

impl ImpellerRenderer {
    pub(crate) fn new<S: HasWindowHandle + HasDisplayHandle>(
        surface: &S,
    ) -> Result<Self, InitError> {
        let window = WaylandSurfacePointers::from_window(surface)?;
        Ok(Self::with_target(RenderTarget::create(window)?))
    }

    pub(crate) fn offscreen(width: u32, height: u32) -> Result<Self, InitError> {
        Ok(Self::with_target(RenderTarget::Offscreen(
            OffscreenTarget::new(width, height)?,
        )))
    }

    pub(crate) fn on_render_node(
        path: &std::path::Path,
        width: u32,
        height: u32,
    ) -> Result<Self, InitError> {
        Ok(Self::with_target(RenderTarget::Dmabuf(DmabufTarget::new(
            path, width, height,
        )?)))
    }

    /// Makes the dma-buf `key` a buffer frames can be rendered into (see
    /// [`ImpellerRenderer::select_dmabuf`]). Only renderers made for a [`crate::RenderNode`]
    /// take dma-bufs.
    pub fn import_dmabuf(
        &mut self,
        key: u64,
        image: &DmabufImage<'_>,
    ) -> Result<(), GraphicsError> {
        self.activate();
        match &mut self.target {
            RenderTarget::Dmabuf(target) => target.import(key, image),
            _ => Err(frame_error("import_dmabuf needs a render-node renderer")),
        }
    }

    /// Drops the import of `key`.
    pub fn forget_dmabuf(&mut self, key: u64) {
        self.activate();
        if let RenderTarget::Dmabuf(target) = &mut self.target {
            target.forget(key);
        }
    }

    /// Where the next frame lands; false if `key` was never imported.
    pub fn select_dmabuf(&mut self, key: u64) -> bool {
        match &mut self.target {
            RenderTarget::Dmabuf(target) => target.select(key),
            _ => false,
        }
    }

    fn with_target(target: RenderTarget) -> Self {
        Self {
            resources: DrawResources::with_shape_program(),
            boundaries: HashMap::new(),
            target,
            viewport: Viewport::default(),
            corners: CornerShape::default(),
            frames_presented: 0,
        }
    }

    /// The last rendered frame as straight-alpha RGBA8, top row first. Only offscreen renderers
    /// (see [`crate::Offscreen`]) keep their frames readable.
    pub fn read_pixels(&self) -> Result<Pixels, GraphicsError> {
        self.target.activate().map_err(frame_error)?;
        let (width, height, rgba) = self
            .target
            .read_rgba()
            .ok_or_else(|| frame_error("read_pixels needs an offscreen renderer"))?;
        Ok(Pixels::from_gl_readback(width, height, &rgba))
    }

    /// The GPU API frames are presented with.
    pub fn api(&self) -> RenderApi {
        self.target.api()
    }

    /// Frames actually put on screen (skipped frames are not counted).
    pub fn frames_presented(&self) -> u64 {
        self.frames_presented
    }

    /// Size in device pixels of the buffers being presented: the live swapchain extent on
    /// Vulkan, the wrapped framebuffer size on GLES.
    pub fn presented_size(&self) -> Option<(u32, u32)> {
        self.target.presented_size()
    }

    /// The viewport last passed to [`Renderer::set_viewport`].
    pub fn viewport(&self) -> (u32, u32, f32) {
        (
            self.viewport.width,
            self.viewport.height,
            self.viewport.scale,
        )
    }

    fn activate(&self) {
        if let Err(error) = self.target.activate() {
            tracing::warn!(%error, "making the EGL context current failed");
        }
    }
}

impl Renderer for ImpellerRenderer {
    type TextLayout = ImpellerTextLayout;

    fn set_viewport(&mut self, width: u32, height: u32, scale: f32) {
        self.viewport = Viewport {
            width,
            height,
            scale,
        };
        if !self.viewport.is_empty() {
            self.target.resize(width, height);
        }
    }

    fn set_corner_shape(&mut self, shape: CornerShape) {
        self.corners = shape;
    }

    fn upload_text(&mut self, id: TextLayoutId, layout: ImpellerTextLayout) {
        match layout.into_paragraph() {
            Some(paragraph) => self.resources.paragraphs.insert(id, paragraph),
            None => self.resources.paragraphs.remove(&id),
        };
    }

    fn release_text(&mut self, id: TextLayoutId) {
        self.resources.paragraphs.remove(&id);
    }

    fn upload_image(
        &mut self,
        id: ImageId,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> Result<(), GraphicsError> {
        let expected = width as usize * height as usize * 4;
        if rgba.len() != expected {
            return Err(GraphicsError::Image(format!(
                "expected {expected} bytes for {width}x{height}, got {}",
                rgba.len()
            )));
        }
        self.activate();
        let pixels = Cow::Owned(premultiplied_rgba(rgba));
        // SAFETY: `Drop` releases every texture before the context.
        let texture = unsafe {
            self.target
                .context()
                .create_texture_with_rgba8(pixels, width, height)
        }
        .map_err(|message| GraphicsError::Image(message.to_owned()))?;
        self.resources.textures.insert(
            id,
            UploadedTexture {
                texture,
                width,
                height,
            },
        );
        Ok(())
    }

    fn release_image(&mut self, id: ImageId) {
        self.activate();
        self.resources.textures.remove(&id);
    }

    fn record_boundary(&mut self, id: BoundaryId, list: &DrawList) {
        let shader = self.resources.shape_shader(self.target.context());
        match translate_draw_list(list, &self.resources, shader, self.corners) {
            Some(display_list) => {
                self.boundaries.insert(id, display_list);
            }
            None => {
                tracing::warn!(boundary = id.0, "Impeller failed to build a display list");
                self.boundaries.remove(&id);
            }
        }
    }

    fn release_boundary(&mut self, id: BoundaryId) {
        self.boundaries.remove(&id);
    }

    fn render(&mut self, composition: &Composition) -> Result<(), GraphicsError> {
        if self.viewport.is_empty() {
            return Ok(());
        }
        let shader = self.resources.shape_shader(self.target.context());
        let frame = compose_frame(
            composition,
            &self.boundaries,
            self.viewport,
            self.corners,
            shader,
        )
        .ok_or_else(|| frame_error("Impeller failed to build the frame display list"))?;
        self.target.activate().map_err(frame_error)?;
        if self.target.present(&frame)? {
            self.frames_presented += 1;
        }
        Ok(())
    }
}

impl Drop for ImpellerRenderer {
    fn drop(&mut self) {
        self.activate();
        self.target.release_swapchain();
        self.boundaries.clear();
        self.resources.clear();
    }
}
