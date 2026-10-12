use geometry::{Color, CornerShape};
use view::ColorScheme;
use windowing::{LayerOptions, Material, WindowAttributes, dpi::LogicalSize};

/// How an app's window starts.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Window title, also the app name for MCP and accessibility.
    pub title: String,
    /// Application id.
    pub app_id: Option<String>,
    /// Initial content size.
    pub size: LogicalSize,
    /// The smallest content size the window may be resized to.
    pub min_size: Option<LogicalSize>,
    /// Platform material behind the whole window.
    pub material: Option<Material>,
    /// Color the window is cleared to; transparent shows the material.
    pub clear_color: Color,
    /// Initial color scheme.
    pub color_scheme: ColorScheme,
    /// Fonts to register before the first layout: bytes and an optional family alias.
    pub fonts: Vec<(Vec<u8>, Option<String>)>,
    /// Serve the semantic tree and actions to agents over MCP.
    pub mcp: bool,
    /// Expose the semantic tree to screen readers.
    pub accessibility: bool,
    /// Run as a desktop-shell layer surface (panel, dock, overlay) instead of a window.
    pub layer: Option<LayerOptions>,
    /// The curve every rounded corner in the window follows.
    pub corner_shape: CornerShape,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            title: "CrownUI".into(),
            app_id: None,
            size: LogicalSize::new(800, 600),
            min_size: None,
            material: None,
            clear_color: geometry::WHITE,
            color_scheme: ColorScheme::Light,
            fonts: Vec::new(),
            mcp: false,
            accessibility: true,
            layer: None,
            corner_shape: CornerShape::Round,
        }
    }
}

impl AppConfig {
    /// A config titled `title`.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Self::default()
        }
    }

    /// Sets the initial size.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.size = LogicalSize::new(width, height);
        self
    }

    /// Keeps the window at least `width` × `height`, for layouts that clip below it.
    pub fn min_size(mut self, width: u32, height: u32) -> Self {
        self.min_size = Some(LogicalSize::new(width, height));
        self
    }

    /// Sets the application id.
    pub fn app_id(mut self, app_id: impl Into<String>) -> Self {
        self.app_id = Some(app_id.into());
        self
    }

    /// Puts a platform material behind the window and clears to transparent.
    pub fn material(mut self, material: Material) -> Self {
        self.material = Some(material);
        self.clear_color = geometry::TRANSPARENT;
        self
    }

    /// Sets the clear color.
    pub fn clear_color(mut self, color: Color) -> Self {
        self.clear_color = color;
        self
    }

    /// Starts in `scheme`.
    pub fn color_scheme(mut self, scheme: ColorScheme) -> Self {
        self.color_scheme = scheme;
        self
    }

    /// Registers a font.
    pub fn font(mut self, bytes: Vec<u8>, alias: Option<&str>) -> Self {
        self.fonts.push((bytes, alias.map(str::to_owned)));
        self
    }

    /// Enables the in-app MCP server.
    pub fn mcp(mut self, enabled: bool) -> Self {
        self.mcp = enabled;
        self
    }

    /// Enables or disables the screen-reader bridge.
    pub fn accessibility(mut self, enabled: bool) -> Self {
        self.accessibility = enabled;
        self
    }

    /// Runs as a layer surface; `size` stays the fallback until the output's size is known.
    pub fn layer(mut self, layer: LayerOptions) -> Self {
        self.layer = Some(layer);
        self
    }

    /// Draws every rounded corner in the window, kit widgets' included, as `shape`.
    pub fn corner_shape(mut self, shape: CornerShape) -> Self {
        self.corner_shape = shape;
        self
    }

    pub(crate) fn window_attributes(&self) -> WindowAttributes {
        let mut attributes = WindowAttributes::default()
            .with_title(self.title.clone())
            .with_inner_size(self.size);
        if let Some(app_id) = &self.app_id {
            attributes = attributes.with_app_id(app_id.clone());
        }
        if let Some(material) = self.material {
            attributes = attributes.with_material(material);
        }
        if let Some(min_size) = self.min_size {
            attributes = attributes.with_min_size(min_size);
        }
        if let Some(layer) = &self.layer {
            attributes = attributes.with_layer(layer.clone());
        }
        attributes
    }
}
