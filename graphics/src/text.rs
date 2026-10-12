use std::borrow::Cow;

use geometry::{Color, Size};

use crate::GraphicsError;

/// Font weight on the CSS 100–900 scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FontWeight(pub u16);

impl FontWeight {
    /// 400.
    pub const NORMAL: Self = Self(400);
    /// 500.
    pub const MEDIUM: Self = Self(500);
    /// 600.
    pub const SEMIBOLD: Self = Self(600);
    /// 700.
    pub const BOLD: Self = Self(700);
}

/// Horizontal alignment of lines, as in CSS `text-align`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum TextAlign {
    /// Leading edge (left in LTR).
    #[default]
    Start,
    /// Centered.
    Center,
    /// Trailing edge.
    End,
    /// Stretched to both edges.
    Justify,
}

/// Everything that affects how text is shaped and drawn.
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    /// Font family name; `None` uses the backend default.
    pub font_family: Option<Cow<'static, str>>,
    /// Size in logical pixels.
    pub font_size: f32,
    /// Weight.
    pub font_weight: FontWeight,
    /// Italic.
    pub italic: bool,
    /// Text color.
    pub color: Color,
    /// Line height as a multiple of the font size (CSS `line-height` without units).
    pub line_height: Option<f32>,
    /// Extra spacing between letters in logical pixels.
    pub letter_spacing: f32,
    /// Alignment.
    pub align: TextAlign,
    /// Maximum number of lines before truncating.
    pub max_lines: Option<u32>,
    /// Whether truncated text ends with an ellipsis.
    pub ellipsis: bool,
    /// Right-to-left base direction.
    pub rtl: bool,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family: None,
            font_size: 14.0,
            font_weight: FontWeight::NORMAL,
            italic: false,
            color: geometry::BLACK,
            line_height: None,
            letter_spacing: 0.0,
            align: TextAlign::Start,
            max_lines: None,
            ellipsis: false,
            rtl: false,
        }
    }
}

impl TextStyle {
    /// A stable hash of everything that affects shaping, for caches.
    pub fn shaping_key(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.font_family.hash(&mut hasher);
        self.font_size.to_bits().hash(&mut hasher);
        self.font_weight.hash(&mut hasher);
        self.italic.hash(&mut hasher);
        self.color.to_rgba8().hash(&mut hasher);
        self.line_height.map(f32::to_bits).hash(&mut hasher);
        self.letter_spacing.to_bits().hash(&mut hasher);
        self.align.hash(&mut hasher);
        self.max_lines.hash(&mut hasher);
        self.ellipsis.hash(&mut hasher);
        self.rtl.hash(&mut hasher);
        hasher.finish()
    }
}

/// Identifies a shaped text layout across the UI and render threads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TextLayoutId(pub u64);

/// Shaped, line-broken text.
pub trait TextLayout {
    /// Size of the laid-out text.
    fn size(&self) -> Size;
    /// Width of the widest unbreakable run (CSS min-content).
    fn min_intrinsic_width(&self) -> f32;
    /// Width without any line breaks (CSS max-content).
    fn max_intrinsic_width(&self) -> f32;
    /// Number of lines.
    fn line_count(&self) -> usize;
    /// Distance from the top to the first baseline.
    fn first_baseline(&self) -> f32;
    /// Approximate heap size, for cache budgets.
    fn memory_bytes(&self) -> usize;
}

/// Shapes text on the UI thread.
pub trait TextSystem {
    /// The layout type, moved to the render thread for drawing.
    type Layout: TextLayout + Send + 'static;

    /// Makes a font available under its own family name and, optionally, `alias`.
    fn register_font(&mut self, bytes: Vec<u8>, alias: Option<&str>) -> Result<(), GraphicsError>;

    /// Shapes `text`, breaking lines at `max_width` when given.
    fn layout(&mut self, text: &str, style: &TextStyle, max_width: Option<f32>) -> Self::Layout;
}
