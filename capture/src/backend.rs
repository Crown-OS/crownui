use graphics::{CreateRenderer, GraphicsBackend, GraphicsError};
use impeller::{ImpellerBackend, Offscreen};

use graphics::TextSystem;

use crate::{
    CaptureRenderer, CaptureTextSystem, CapturedText,
    renderer_pool::take_spare,
    window_look::{take_fonts, take_look},
};

/// Impeller rendering offscreen, with every frame kept for [`crate::capture`].
#[derive(Debug, Clone, Copy, Default)]
pub struct CaptureBackend;

impl GraphicsBackend for CaptureBackend {
    type TextLayout = CapturedText;
    type TextSystem = CaptureTextSystem;
    type Renderer = CaptureRenderer;

    fn create_text_system() -> Result<CaptureTextSystem, GraphicsError> {
        let mut text = CaptureTextSystem::default();
        for (bytes, alias) in take_fonts() {
            text.register_font(bytes, alias.as_deref())?;
        }
        Ok(text)
    }
}

impl CreateRenderer<Offscreen> for CaptureBackend {
    fn create_renderer(surface: &Offscreen) -> Result<CaptureRenderer, GraphicsError> {
        let size = (surface.width, surface.height);
        let gpu = match take_spare(size) {
            Some(gpu) => gpu,
            None => ImpellerBackend::create_renderer(surface)?,
        };
        Ok(CaptureRenderer::new(gpu, size, take_look()))
    }
}
