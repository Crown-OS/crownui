use std::collections::HashMap;

use graphics::{ImageId, TextLayoutId};
use impellers::{Context, FragmentProgram, Paragraph, Texture};

use crate::util::shape_shader::{ShapeShader, load_shape_program};

/// A GPU texture and its size in texels.
#[derive(Debug)]
pub(crate) struct UploadedTexture {
    pub(crate) texture: Texture,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

/// Paragraphs and textures that draw lists refer to by id, and the rounded-shape program.
#[derive(Debug, Default)]
pub(crate) struct DrawResources {
    pub(crate) paragraphs: HashMap<TextLayoutId, Paragraph>,
    pub(crate) textures: HashMap<ImageId, UploadedTexture>,
    pub(crate) shape_program: Option<FragmentProgram>,
}

impl DrawResources {
    pub(crate) fn with_shape_program() -> Self {
        Self {
            shape_program: load_shape_program(),
            ..Self::default()
        }
    }

    /// The rounded-shape shader on `context`, when its program loaded.
    pub(crate) fn shape_shader<'gpu>(
        &'gpu self,
        context: &'gpu Context,
    ) -> Option<ShapeShader<'gpu>> {
        self.shape_program
            .as_ref()
            .map(|program| ShapeShader::new(context, program))
    }

    pub(crate) fn clear(&mut self) {
        self.paragraphs.clear();
        self.textures.clear();
        self.shape_program = None;
    }
}
