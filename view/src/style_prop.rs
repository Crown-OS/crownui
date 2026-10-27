use geometry::{Color, CornerRadii, Vector};
use graphics::{FontWeight, TextAlign};
use layouting::{Align, Dimension, Display, FlexDirection, GridTrack, Justify, Overflow, Position};
use windowing::{CursorIcon, Material};

use crate::style::{NodeStyle, Shadow};

/// Which part of a node a style change invalidates.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StyleImpact {
    /// Layout must be recomputed.
    pub layout: bool,
    /// The node must be repainted or recomposited.
    pub visual: bool,
    /// Text must be reshaped (in this subtree).
    pub text: bool,
}

/// Edges a box-model property applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edges {
    /// All four.
    All,
    /// Left and right.
    Horizontal,
    /// Top and bottom.
    Vertical,
    /// Top only.
    Top,
    /// Right only.
    Right,
    /// Bottom only.
    Bottom,
    /// Left only.
    Left,
}

/// Corners a radius applies to, like Tailwind's `rounded-t`, `rounded-tl` …
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Corners {
    All,
    Top,
    Right,
    Bottom,
    Left,
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

/// Axis of a size property.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum SizeProperty {
    Width,
    Height,
    MinWidth,
    MinHeight,
    MaxWidth,
    MaxHeight,
}

/// One style assignment; modifiers, variants and bindings all produce these.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum StyleProp {
    Padding(Edges, f32),
    Margin(Edges, f32),
    Size(SizeProperty, Dimension),
    AspectRatio(f32),
    Display(Display),
    Position(Position),
    Inset(Edges, f32),
    Gap(Option<f32>, Option<f32>),
    FlexDirection(FlexDirection),
    FlexWrap(bool),
    Grow(f32),
    Shrink(f32),
    Basis(Dimension),
    AlignItems(Align),
    AlignSelf(Align),
    Justify(Justify),
    GridColumns(Vec<GridTrack>),
    ColumnSpan(u16),
    RowSpan(u16),
    Overflow(Overflow),
    Background(Option<Color>),
    BorderWidth(f32),
    BorderColor(Color),
    Ring(Option<(f32, Color)>),
    Radius(Corners, f32),
    Shadow(Option<Shadow>),
    Opacity(f32),
    Translate(Vector),
    Scale(f32),
    ScaleX(f32),
    ScaleY(f32),
    Rotate(f32),
    Origin(Vector),
    BackdropBlur(f32),
    Hidden(bool),
    ZIndex(i16),
    TopLayer(bool),
    ContentOffset(Vector),
    FontFamily(String),
    TextSize(f32),
    FontWeight(FontWeight),
    Italic(bool),
    TextColor(Color),
    LineHeight(f32),
    Tracking(f32),
    TextAlign(TextAlign),
    LineClamp(Option<u32>),
    Truncate(bool),
    Cursor(CursorIcon),
    Material(Option<Material>),
    PointerEvents(crate::PointerEvents),
}

fn apply_edges(insets: &mut geometry::Insets, edges: Edges, value: f32) {
    match edges {
        Edges::All => *insets = geometry::Insets::uniform(value),
        Edges::Horizontal => (insets.left, insets.right) = (value, value),
        Edges::Vertical => (insets.top, insets.bottom) = (value, value),
        Edges::Top => insets.top = value,
        Edges::Right => insets.right = value,
        Edges::Bottom => insets.bottom = value,
        Edges::Left => insets.left = value,
    }
}

fn apply_corners(radii: &mut CornerRadii, corners: Corners, value: f32) {
    let (top_left, top_right, bottom_right, bottom_left) = match corners {
        Corners::All => (true, true, true, true),
        Corners::Top => (true, true, false, false),
        Corners::Right => (false, true, true, false),
        Corners::Bottom => (false, false, true, true),
        Corners::Left => (true, false, false, true),
        Corners::TopLeft => (true, false, false, false),
        Corners::TopRight => (false, true, false, false),
        Corners::BottomRight => (false, false, true, false),
        Corners::BottomLeft => (false, false, false, true),
    };
    let targets = [
        (top_left, &mut radii.top_left),
        (top_right, &mut radii.top_right),
        (bottom_right, &mut radii.bottom_right),
        (bottom_left, &mut radii.bottom_left),
    ];
    for (selected, radius) in targets {
        if selected {
            *radius = value;
        }
    }
}

impl StyleProp {
    /// Whether this property may appear in a state variant (`hover`, `dark` …).
    pub fn is_visual(&self) -> bool {
        self.impact()
            == StyleImpact {
                layout: false,
                visual: true,
                text: false,
            }
    }

    /// What applying this property invalidates.
    pub fn impact(&self) -> StyleImpact {
        let layout = StyleImpact {
            layout: true,
            visual: false,
            text: false,
        };
        let visual = StyleImpact {
            layout: false,
            visual: true,
            text: false,
        };
        let text = StyleImpact {
            layout: true,
            visual: true,
            text: true,
        };
        match self {
            Self::Padding(..)
            | Self::Margin(..)
            | Self::Size(..)
            | Self::AspectRatio(_)
            | Self::Display(_)
            | Self::Position(_)
            | Self::Inset(..)
            | Self::Gap(..)
            | Self::FlexDirection(_)
            | Self::FlexWrap(_)
            | Self::Grow(_)
            | Self::Shrink(_)
            | Self::Basis(_)
            | Self::AlignItems(_)
            | Self::AlignSelf(_)
            | Self::Justify(_)
            | Self::GridColumns(_)
            | Self::ColumnSpan(_)
            | Self::RowSpan(_) => layout,
            Self::Overflow(_) => StyleImpact {
                layout: true,
                visual: true,
                text: false,
            },
            Self::FontFamily(_)
            | Self::TextSize(_)
            | Self::FontWeight(_)
            | Self::Italic(_)
            | Self::TextColor(_)
            | Self::LineHeight(_)
            | Self::Tracking(_)
            | Self::TextAlign(_)
            | Self::LineClamp(_)
            | Self::Truncate(_) => text,
            Self::Background(_)
            | Self::BorderWidth(_)
            | Self::BorderColor(_)
            | Self::Ring(_)
            | Self::Radius(..)
            | Self::Shadow(_)
            | Self::Opacity(_)
            | Self::Translate(_)
            | Self::Scale(_)
            | Self::ScaleX(_)
            | Self::ScaleY(_)
            | Self::Rotate(_)
            | Self::Origin(_)
            | Self::BackdropBlur(_)
            | Self::Hidden(_)
            | Self::ZIndex(_)
            | Self::TopLayer(_)
            | Self::ContentOffset(_)
            | Self::Cursor(_)
            | Self::Material(_)
            | Self::PointerEvents(_) => visual,
        }
    }

    /// Writes this property into `style`.
    pub fn apply(&self, style: &mut NodeStyle) {
        let layout = &mut style.layout;
        let visual = &mut style.visual;
        let text = &mut style.text;
        match self {
            Self::Padding(edges, value) => apply_edges(&mut layout.padding, *edges, *value),
            Self::Margin(edges, value) => apply_edges(&mut layout.margin, *edges, *value),
            Self::Size(property, dimension) => match property {
                SizeProperty::Width => layout.width = *dimension,
                SizeProperty::Height => layout.height = *dimension,
                SizeProperty::MinWidth => layout.min_width = *dimension,
                SizeProperty::MinHeight => layout.min_height = *dimension,
                SizeProperty::MaxWidth => layout.max_width = *dimension,
                SizeProperty::MaxHeight => layout.max_height = *dimension,
            },
            Self::AspectRatio(ratio) => layout.aspect_ratio = Some(*ratio),
            Self::Display(display) => layout.display = *display,
            Self::Position(position) => layout.position = *position,
            Self::Inset(edges, value) => {
                let mut insets = geometry::Insets::ZERO;
                apply_edges(&mut insets, *edges, *value);
                let sides = [
                    matches!(edges, Edges::All | Edges::Vertical | Edges::Top),
                    matches!(edges, Edges::All | Edges::Horizontal | Edges::Right),
                    matches!(edges, Edges::All | Edges::Vertical | Edges::Bottom),
                    matches!(edges, Edges::All | Edges::Horizontal | Edges::Left),
                ];
                let values = [insets.top, insets.right, insets.bottom, insets.left];
                for ((slot, selected), value) in layout.inset.iter_mut().zip(sides).zip(values) {
                    if selected {
                        *slot = Dimension::Length(value);
                    }
                }
            }
            Self::Gap(column, row) => {
                if let Some(column) = column {
                    layout.gap.0 = *column;
                }
                if let Some(row) = row {
                    layout.gap.1 = *row;
                }
            }
            Self::FlexDirection(direction) => layout.flex_direction = *direction,
            Self::FlexWrap(wrap) => layout.flex_wrap = *wrap,
            Self::Grow(grow) => layout.flex_grow = *grow,
            Self::Shrink(shrink) => layout.flex_shrink = *shrink,
            Self::Basis(basis) => layout.flex_basis = *basis,
            Self::AlignItems(align) => layout.align_items = Some(*align),
            Self::AlignSelf(align) => layout.align_self = Some(*align),
            Self::Justify(justify) => layout.justify_content = Some(*justify),
            Self::GridColumns(tracks) => {
                layout.display = Display::Grid;
                layout.grid_columns.clone_from(tracks);
            }
            Self::ColumnSpan(span) => layout.column_span = *span,
            Self::RowSpan(span) => layout.row_span = *span,
            Self::Overflow(overflow) => {
                layout.overflow = *overflow;
                visual.clip = *overflow != Overflow::Visible;
            }
            Self::Background(color) => visual.background = *color,
            Self::BorderWidth(width) => visual.border_width = *width,
            Self::BorderColor(color) => visual.border_color = *color,
            Self::Ring(ring) => visual.ring = *ring,
            Self::Radius(corners, value) => apply_corners(&mut visual.radii, *corners, *value),
            Self::Shadow(shadow) => visual.shadow = *shadow,
            Self::Opacity(opacity) => visual.opacity = opacity.clamp(0.0, 1.0),
            Self::Translate(offset) => visual.translate = *offset,
            Self::Scale(scale) => visual.scale = Vector::new(*scale, *scale),
            Self::ScaleX(scale) => visual.scale.x = *scale,
            Self::ScaleY(scale) => visual.scale.y = *scale,
            Self::Rotate(degrees) => visual.rotate = *degrees,
            Self::Origin(origin) => visual.origin = *origin,
            Self::BackdropBlur(radius) => visual.backdrop_blur = radius.max(0.0),
            Self::Hidden(hidden) => visual.hidden = *hidden,
            Self::ZIndex(z) => visual.z_index = *z,
            Self::TopLayer(top_layer) => visual.top_layer = *top_layer,
            Self::ContentOffset(offset) => visual.content_offset = *offset,
            Self::FontFamily(family) => text.font_family = Some(family.clone()),
            Self::TextSize(size) => text.font_size = Some(*size),
            Self::FontWeight(weight) => text.font_weight = Some(*weight),
            Self::Italic(italic) => text.italic = Some(*italic),
            Self::TextColor(color) => text.color = Some(*color),
            Self::LineHeight(height) => text.line_height = Some(*height),
            Self::Tracking(spacing) => text.letter_spacing = Some(*spacing),
            Self::TextAlign(align) => text.align = Some(*align),
            Self::LineClamp(lines) => text.max_lines = *lines,
            Self::Truncate(truncate) => {
                text.ellipsis = Some(*truncate);
                if *truncate {
                    text.max_lines = Some(1);
                }
            }
            Self::Cursor(cursor) => style.cursor = Some(*cursor),
            Self::Material(material) => style.material = *material,
            Self::PointerEvents(events) => style.pointer_events = Some(*events),
        }
    }
}
