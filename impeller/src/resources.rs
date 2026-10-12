use std::collections::HashMap;

use graphics::{ImageId, TextLayoutId};
use impellers::{Paragraph, Texture};

/// A GPU texture and its size in texels.
#[derive(Debug)]
pub(crate) struct UploadedTexture {
    pub(crate) texture: Texture,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

/// Paragraphs and textures that draw lists refer to by id.
#[derive(Debug, Default)]
pub(crate) struct DrawResources {
    pub(crate) paragraphs: HashMap<TextLayoutId, Paragraph>,
    pub(crate) textures: HashMap<ImageId, UploadedTexture>,
}

impl DrawResources {
    pub(crate) fn clear(&mut self) {
        self.paragraphs.clear();
        self.textures.clear();
    }
}
