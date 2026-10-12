use graphics::GraphicsError;

/// Why creating a renderer failed.
#[derive(Debug, thiserror::Error)]
pub(crate) enum InitError {
    #[error(
        "unsupported window handle {0}; the Impeller backend presents only to Wayland surfaces"
    )]
    UnsupportedWindow(String),
    #[error("unsupported display handle {0}; the Impeller backend needs a Wayland display")]
    UnsupportedDisplay(String),
    #[error("window handle unavailable: {0}")]
    Handle(raw_window_handle::HandleError),
    #[error("Vulkan loader: {0}")]
    VulkanLoader(#[from] ash::LoadingError),
    #[error("Vulkan: {0}")]
    Vulkan(#[from] ash::vk::Result),
    #[error("all {0} Vulkan surface extent slots are in use")]
    SurfaceSlotsExhausted(usize),
    #[error("EGL: {0}")]
    Egl(#[from] khronos_egl::Error),
    #[error("no RGBA8 + stencil8 GLES3 EGL config")]
    NoEglConfig,
    #[error("EGL extension {0} is missing")]
    MissingEglExtension(&'static str),
    #[error("no default EGL display")]
    NoEglDisplay,
    #[error("no EGL device drives render node {0}")]
    NoRenderNodeDevice(String),
    #[error("GL entry point {0} is missing")]
    MissingGlFunction(&'static str),
    #[error("GL: {0}")]
    Gl(String),
    #[error("offscreen framebuffer incomplete (status {0:#x})")]
    IncompleteFramebuffer(u32),
    #[error("wayland-egl: {0}")]
    WaylandEgl(#[from] wayland_egl::Error),
    #[error("Impeller: {0}")]
    Impeller(&'static str),
    #[error("Vulkan failed ({vulkan}) and the GLES fallback failed too ({gles})")]
    NoBackend {
        vulkan: Box<InitError>,
        gles: Box<InitError>,
    },
}

impl From<InitError> for GraphicsError {
    fn from(error: InitError) -> Self {
        GraphicsError::Initialization(error.to_string())
    }
}
