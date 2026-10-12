use graphics::{CreateRenderer, GraphicsBackend, GraphicsError};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::{ImpellerRenderer, ImpellerTextLayout, ImpellerTextSystem};

/// The Impeller graphics backend: Impeller typography for text, Vulkan or GLES for drawing.
#[derive(Debug, Clone, Copy, Default)]
pub struct ImpellerBackend;

impl GraphicsBackend for ImpellerBackend {
    type TextLayout = ImpellerTextLayout;
    type TextSystem = ImpellerTextSystem;
    type Renderer = ImpellerRenderer;

    fn create_text_system() -> Result<ImpellerTextSystem, GraphicsError> {
        Ok(ImpellerTextSystem::new())
    }
}

impl<S: HasWindowHandle + HasDisplayHandle> CreateRenderer<S> for ImpellerBackend {
    fn create_renderer(surface: &S) -> Result<ImpellerRenderer, GraphicsError> {
        Ok(ImpellerRenderer::new(surface)?)
    }
}
