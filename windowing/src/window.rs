use crate::{dpi::LogicalSize, layer::LayerOptions, material::Material};

/// Identifies a window within one platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WindowId(u64);

impl WindowId {
    /// Wraps a backend-chosen identifier.
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// The backend-chosen identifier.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Who draws the window frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Decorations {
    /// Ask the platform to draw the title bar and borders.
    #[default]
    Server,
    /// The client draws everything, or the window is undecorated.
    None,
}

/// Pointer cursor shapes, named after CSS `cursor` values.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum CursorIcon {
    #[default]
    Default,
    Pointer,
    Text,
    Grab,
    Grabbing,
    Move,
    NotAllowed,
    Wait,
    Progress,
    Crosshair,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ColResize,
    RowResize,
}

/// Options for a new window.
#[derive(Debug, Clone)]
pub struct WindowAttributes {
    /// Title shown by the platform.
    pub title: String,
    /// Application id used to group windows and pick icons.
    pub app_id: Option<String>,
    /// Initial content size.
    pub inner_size: LogicalSize,
    /// Smallest allowed size.
    pub min_size: Option<LogicalSize>,
    /// Largest allowed size.
    pub max_size: Option<LogicalSize>,
    /// Who draws the frame.
    pub decorations: Decorations,
    /// Start maximized.
    pub maximized: bool,
    /// Platform-rendered background, when supported.
    pub material: Option<Material>,
    /// Makes the window a desktop-shell layer surface (panel, dock, overlay) instead of an
    /// application window; title, decorations and size limits then do not apply.
    pub layer: Option<LayerOptions>,
}

impl Default for WindowAttributes {
    fn default() -> Self {
        Self {
            title: String::new(),
            app_id: None,
            inner_size: LogicalSize::new(800, 600),
            min_size: None,
            max_size: None,
            decorations: Decorations::default(),
            maximized: false,
            material: None,
            layer: None,
        }
    }
}

impl WindowAttributes {
    /// Sets the title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the application id.
    pub fn with_app_id(mut self, app_id: impl Into<String>) -> Self {
        self.app_id = Some(app_id.into());
        self
    }

    /// Sets the initial content size.
    pub fn with_inner_size(mut self, size: LogicalSize) -> Self {
        self.inner_size = size;
        self
    }

    /// Sets the smallest allowed size.
    pub fn with_min_size(mut self, size: LogicalSize) -> Self {
        self.min_size = Some(size);
        self
    }

    /// Sets the largest allowed size.
    pub fn with_max_size(mut self, size: LogicalSize) -> Self {
        self.max_size = Some(size);
        self
    }

    /// Chooses who draws the frame.
    pub fn with_decorations(mut self, decorations: Decorations) -> Self {
        self.decorations = decorations;
        self
    }

    /// Starts maximized.
    pub fn with_maximized(mut self) -> Self {
        self.maximized = true;
        self
    }

    /// Requests a platform-rendered background; ignored where unsupported.
    pub fn with_material(mut self, material: Material) -> Self {
        self.material = Some(material);
        self
    }

    /// Makes the window a layer surface; ignored where the platform has no layer shell.
    pub fn with_layer(mut self, layer: LayerOptions) -> Self {
        self.layer = Some(layer);
        self
    }
}
