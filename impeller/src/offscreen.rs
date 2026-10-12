use graphics::{CreateRenderer, GraphicsError, Renderer};

use crate::{ImpellerBackend, ImpellerRenderer};

/// A windowless render target: frames go to an offscreen GLES framebuffer and are read back
/// with [`ImpellerRenderer::read_pixels`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Offscreen {
    /// Width in device pixels.
    pub width: u32,
    /// Height in device pixels.
    pub height: u32,
    /// Logical-to-device scale.
    pub scale: f32,
}

impl CreateRenderer<Offscreen> for ImpellerBackend {
    fn create_renderer(offscreen: &Offscreen) -> Result<ImpellerRenderer, GraphicsError> {
        let mut renderer = ImpellerRenderer::offscreen(offscreen.width, offscreen.height)?;
        renderer.set_viewport(offscreen.width, offscreen.height, offscreen.scale);
        Ok(renderer)
    }
}
