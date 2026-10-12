use geometry::CornerShape;

use crate::{BoundaryId, Composition, DrawList, ImageId, TextLayout, TextLayoutId, TextSystem};

/// Why a graphics operation failed.
#[derive(Debug, thiserror::Error)]
pub enum GraphicsError {
    /// The GPU context or surface could not be created.
    #[error("graphics initialization failed: {0}")]
    Initialization(String),
    /// Presenting or drawing a frame failed.
    #[error("frame failed: {0}")]
    Frame(String),
    /// A font could not be registered.
    #[error("font rejected: {0}")]
    Font(String),
    /// An image could not be uploaded.
    #[error("image rejected: {0}")]
    Image(String),
}

/// Draws compositions on the render thread, caching recorded boundaries.
pub trait Renderer {
    /// The text layouts this renderer draws.
    type TextLayout;

    /// Sets the drawable size in device pixels and the logical-to-device scale.
    fn set_viewport(&mut self, width: u32, height: u32, scale: f32);
    /// Draws every rounded rectangle, clip and shadow recorded from now on with `shape` corners.
    /// Renderers that only draw one shape ignore it.
    fn set_corner_shape(&mut self, _shape: CornerShape) {}
    /// Makes a text layout drawable under `id`.
    fn upload_text(&mut self, id: TextLayoutId, layout: Self::TextLayout);
    /// Forgets a text layout.
    fn release_text(&mut self, id: TextLayoutId);
    /// Uploads straight-alpha RGBA8 pixels under `id`.
    fn upload_image(
        &mut self,
        id: ImageId,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> Result<(), GraphicsError>;
    /// Forgets an image.
    fn release_image(&mut self, id: ImageId);
    /// Records (or re-records) a boundary's list.
    fn record_boundary(&mut self, id: BoundaryId, list: &DrawList);
    /// Forgets a boundary.
    fn release_boundary(&mut self, id: BoundaryId);
    /// Draws and presents one frame.
    fn render(&mut self, composition: &Composition) -> Result<(), GraphicsError>;
}

/// A text system and a renderer that agree on the layout type.
pub trait GraphicsBackend: 'static {
    /// Shaped text, created on the UI thread and drawn on the render thread.
    type TextLayout: TextLayout + Send + 'static;
    /// UI-thread text shaping.
    type TextSystem: TextSystem<Layout = Self::TextLayout>;
    /// Render-thread drawing.
    type Renderer: Renderer<TextLayout = Self::TextLayout>;

    /// Creates the UI thread's text system.
    fn create_text_system() -> Result<Self::TextSystem, GraphicsError>;
}

/// Creates a renderer presenting to surfaces of type `S`.
pub trait CreateRenderer<S>: GraphicsBackend {
    /// Creates a renderer for `surface`; the caller keeps `surface` alive longer than the renderer.
    fn create_renderer(surface: &S) -> Result<Self::Renderer, GraphicsError>;
}
