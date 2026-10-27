use geometry::{Color, CornerRadii, Vector};
use graphics::{FontWeight, TextAlign};
use layouting::LayoutStyle;
use windowing::{CursorIcon, Material};

use crate::PointerEvents;

/// A CSS-style `box-shadow`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
    /// Shadow color.
    pub color: Color,
    /// Blur radius in logical pixels.
    pub blur: f32,
    /// Offset from the box.
    pub offset: Vector,
}

impl Shadow {
    /// Tailwind's `shadow-sm`.
    pub fn sm() -> Self {
        Self::new(Color::new(0.0, 0.0, 0.0, 0.05), 2.0, Vector::new(0.0, 1.0))
    }

    /// Tailwind's `shadow-md`.
    pub fn md() -> Self {
        Self::new(Color::new(0.0, 0.0, 0.0, 0.1), 6.0, Vector::new(0.0, 4.0))
    }

    /// Tailwind's `shadow-lg`.
    pub fn lg() -> Self {
        Self::new(Color::new(0.0, 0.0, 0.0, 0.1), 15.0, Vector::new(0.0, 10.0))
    }

    /// Tailwind's `shadow-xl`.
    pub fn xl() -> Self {
        Self::new(Color::new(0.0, 0.0, 0.0, 0.1), 25.0, Vector::new(0.0, 20.0))
    }

    /// A shadow with explicit parameters.
    pub const fn new(color: Color, blur: f32, offset: Vector) -> Self {
        Self {
            color,
            blur,
            offset,
        }
    }
}

/// Everything about how a node looks, apart from layout and content. Every numeric and color
/// field animates on the render thread.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Visual {
    /// Fill behind the content.
    pub background: Option<Color>,
    /// Border width.
    pub border_width: f32,
    /// Border color.
    pub border_color: Color,
    /// Focus ring outside the border: width and color.
    pub ring: Option<(f32, Color)>,
    /// Corner radii, also used to clip children when `clip` is set.
    pub radii: CornerRadii,
    /// Drop shadow.
    pub shadow: Option<Shadow>,
    /// Opacity of the node and its subtree.
    pub opacity: f32,
    /// Translation applied after layout.
    pub translate: Vector,
    /// Horizontal and vertical scale about the transform origin.
    pub scale: Vector,
    /// Clockwise rotation in degrees about the transform origin.
    pub rotate: f32,
    /// Transform origin as a fraction of the size (`0.5, 0.5` is the center).
    pub origin: Vector,
    /// Backdrop blur radius (CSS `backdrop-filter: blur`).
    pub backdrop_blur: f32,
    /// Whether children are clipped to the rounded box.
    pub clip: bool,
    /// Whether the node is hidden (still laid out).
    pub hidden: bool,
    /// Painting order among siblings.
    pub z_index: i16,
    /// Painted and hit-tested above every other node, outside its ancestors' clips.
    pub top_layer: bool,
    /// Offset of children inside a scroll container.
    pub content_offset: Vector,
}

impl Default for Visual {
    fn default() -> Self {
        Self {
            background: None,
            border_width: 0.0,
            border_color: geometry::TRANSPARENT,
            ring: None,
            radii: CornerRadii::ZERO,
            shadow: None,
            opacity: 1.0,
            translate: Vector::ZERO,
            scale: Vector::new(1.0, 1.0),
            rotate: 0.0,
            origin: Vector::new(0.5, 0.5),
            backdrop_blur: 0.0,
            clip: false,
            hidden: false,
            z_index: 0,
            top_layer: false,
            content_offset: Vector::ZERO,
        }
    }
}

impl Visual {
    /// Whether the node itself draws anything (background, border, shadow or ring).
    pub fn paints_box(&self) -> bool {
        self.background.is_some_and(|color| color.a > 0.0)
            || self.border_width > 0.0
            || self.shadow.is_some()
            || self.ring.is_some()
    }
}

/// Text properties inherited down the tree like CSS.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TextOverrides {
    /// `font-family`.
    pub font_family: Option<String>,
    /// `font-size`.
    pub font_size: Option<f32>,
    /// `font-weight`.
    pub font_weight: Option<FontWeight>,
    /// `font-style: italic`.
    pub italic: Option<bool>,
    /// `color`.
    pub color: Option<Color>,
    /// `line-height` multiplier.
    pub line_height: Option<f32>,
    /// `letter-spacing` in logical pixels.
    pub letter_spacing: Option<f32>,
    /// `text-align`.
    pub align: Option<TextAlign>,
    /// `line-clamp`.
    pub max_lines: Option<u32>,
    /// `text-overflow: ellipsis`.
    pub ellipsis: Option<bool>,
}

impl TextOverrides {
    /// Fills unset fields from `parent`.
    pub fn inherit_from(&mut self, parent: &TextOverrides) {
        fn fill<T: Clone>(slot: &mut Option<T>, parent: &Option<T>) {
            if slot.is_none() {
                slot.clone_from(parent);
            }
        }
        fill(&mut self.font_family, &parent.font_family);
        fill(&mut self.font_size, &parent.font_size);
        fill(&mut self.font_weight, &parent.font_weight);
        fill(&mut self.italic, &parent.italic);
        fill(&mut self.color, &parent.color);
        fill(&mut self.line_height, &parent.line_height);
        fill(&mut self.letter_spacing, &parent.letter_spacing);
        fill(&mut self.align, &parent.align);
    }

    /// Resolves into a full text style.
    pub fn resolve(&self, rtl: bool) -> graphics::TextStyle {
        let defaults = graphics::TextStyle::default();
        graphics::TextStyle {
            font_family: self.font_family.clone().map(Into::into),
            font_size: self.font_size.unwrap_or(defaults.font_size),
            font_weight: self.font_weight.unwrap_or(defaults.font_weight),
            italic: self.italic.unwrap_or(false),
            color: self.color.unwrap_or(defaults.color),
            line_height: self.line_height,
            letter_spacing: self.letter_spacing.unwrap_or(0.0),
            align: self.align.unwrap_or_default(),
            max_lines: self.max_lines,
            ellipsis: self.ellipsis.unwrap_or(false),
            rtl,
        }
    }
}

/// The full style of one node.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NodeStyle {
    /// CSS box model and flex/grid properties.
    pub layout: LayoutStyle,
    /// Paint and composite properties before state variants.
    pub visual: Visual,
    /// Text properties set on this node.
    pub text: TextOverrides,
    /// Cursor while hovered.
    pub cursor: Option<CursorIcon>,
    /// Platform material behind this node.
    pub material: Option<Material>,
    /// Whether the node receives pointer events; `None` inherits from the parent.
    pub pointer_events: Option<PointerEvents>,
}
