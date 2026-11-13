use std::{collections::HashMap, mem::ManuallyDrop};

use geometry::CornerShape;
use graphics::{
    BoundaryId, Composition, DrawList, GraphicsError, ImageId, Renderer, TextLayout, TextLayoutId,
};
use impeller::{ImpellerRenderer, Pixels};

use crate::{
    renderer_pool::keep_spare,
    text::{CapturedText, TextSource},
    window_look::WindowLook,
};

/// A text layout as drawn: its source and measured box.
#[derive(Debug, Clone)]
pub(crate) struct DrawnText {
    pub(crate) source: std::sync::Arc<TextSource>,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) lines: usize,
}

/// Renders offscreen with Impeller while mirroring every list, text and image it is given.
pub struct CaptureRenderer {
    gpu: ManuallyDrop<ImpellerRenderer>,
    size: (u32, u32),
    pub(crate) boundaries: HashMap<BoundaryId, DrawList>,
    pub(crate) texts: HashMap<TextLayoutId, DrawnText>,
    pub(crate) images: HashMap<ImageId, Pixels>,
    pub(crate) composition: Composition,
    pub(crate) corner_shape: CornerShape,
    pub(crate) look: WindowLook,
}

impl CaptureRenderer {
    pub(crate) fn new(mut gpu: ImpellerRenderer, size: (u32, u32), look: WindowLook) -> Self {
        gpu.set_corner_shape(look.corner_shape);
        Self {
            gpu: ManuallyDrop::new(gpu),
            size,
            boundaries: HashMap::new(),
            texts: HashMap::new(),
            images: HashMap::new(),
            composition: Composition::default(),
            corner_shape: look.corner_shape,
            look,
        }
    }

    /// The last frame as the GPU drew it.
    pub fn read_pixels(&self) -> Result<Pixels, GraphicsError> {
        self.gpu.read_pixels()
    }
}

impl Drop for CaptureRenderer {
    fn drop(&mut self) {
        for id in self.boundaries.keys() {
            self.gpu.release_boundary(*id);
        }
        for id in self.texts.keys() {
            self.gpu.release_text(*id);
        }
        for id in self.images.keys() {
            self.gpu.release_image(*id);
        }
        // SAFETY: `gpu` is never used again; it moves into the spare pool instead of dropping.
        let gpu = unsafe { ManuallyDrop::take(&mut self.gpu) };
        keep_spare(self.size, gpu);
    }
}

impl Renderer for CaptureRenderer {
    type TextLayout = CapturedText;

    fn set_viewport(&mut self, width: u32, height: u32, scale: f32) {
        self.gpu.set_viewport(width, height, scale);
    }

    fn set_corner_shape(&mut self, shape: CornerShape) {
        self.look.corner_shape = shape;
        self.corner_shape = shape;
        self.gpu.set_corner_shape(shape);
    }

    fn upload_text(&mut self, id: TextLayoutId, layout: CapturedText) {
        let size = layout.size();
        self.texts.insert(
            id,
            DrawnText {
                source: layout.source,
                width: size.width,
                height: size.height,
                lines: layout.layout.line_count(),
            },
        );
        self.gpu.upload_text(id, layout.layout);
    }

    fn release_text(&mut self, id: TextLayoutId) {
        self.texts.remove(&id);
        self.gpu.release_text(id);
    }

    fn upload_image(
        &mut self,
        id: ImageId,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> Result<(), GraphicsError> {
        self.images.insert(
            id,
            Pixels {
                width,
                height,
                rgba: rgba.to_vec(),
            },
        );
        self.gpu.upload_image(id, width, height, rgba)
    }

    fn release_image(&mut self, id: ImageId) {
        self.images.remove(&id);
        self.gpu.release_image(id);
    }

    fn record_boundary(&mut self, id: BoundaryId, list: &DrawList) {
        self.boundaries.insert(id, list.clone());
        self.gpu.record_boundary(id, list);
    }

    fn release_boundary(&mut self, id: BoundaryId) {
        self.boundaries.remove(&id);
        self.gpu.release_boundary(id);
    }

    fn render(&mut self, composition: &Composition) -> Result<(), GraphicsError> {
        self.composition = composition.clone();
        if let Some(clear_color) = self.look.clear_color {
            self.composition.clear_color = clear_color;
        }
        self.gpu.render(&self.composition)
    }
}
