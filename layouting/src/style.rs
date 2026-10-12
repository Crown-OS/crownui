use geometry::Insets;

/// A length that may be automatic or relative to the parent.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Dimension {
    /// Decided by content and the layout algorithm.
    #[default]
    Auto,
    /// Logical pixels.
    Length(f32),
    /// Fraction of the parent's size, `1.0` being 100%.
    Percent(f32),
}

/// CSS `display`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Display {
    /// Flexbox container (also used for leaves).
    #[default]
    Flex,
    /// Grid container.
    Grid,
    /// Removed from layout.
    None,
}

/// CSS `position`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Position {
    /// In normal flow.
    #[default]
    Relative,
    /// Placed by `inset` against the parent, out of flow.
    Absolute,
}

/// CSS `flex-direction`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FlexDirection {
    /// Left to right (start to end).
    Row,
    /// Top to bottom.
    #[default]
    Column,
    /// End to start.
    RowReverse,
    /// Bottom to top.
    ColumnReverse,
}

/// CSS `align-items` / `align-self` / `align-content` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    /// Start edge.
    Start,
    /// Center.
    Center,
    /// End edge.
    End,
    /// Fill the cross axis.
    Stretch,
    /// First baselines aligned.
    Baseline,
}

/// CSS `justify-content` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Justify {
    /// Packed at the start.
    Start,
    /// Centered.
    Center,
    /// Packed at the end.
    End,
    /// Even gaps, none at the edges.
    SpaceBetween,
    /// Even gaps, half at the edges.
    SpaceAround,
    /// Even gaps including the edges.
    SpaceEvenly,
}

/// CSS `overflow`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Overflow {
    /// Children may draw outside.
    #[default]
    Visible,
    /// Children are clipped.
    Clip,
    /// Children are clipped and the content may be larger than the box (scroll containers).
    Scroll,
}

/// One grid track size, as in `grid-template-columns`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridTrack {
    /// Fixed logical pixels.
    Length(f32),
    /// Share of the remaining space (`1fr`).
    Fraction(f32),
    /// Sized to content.
    Auto,
}

/// Everything layout needs to know about one node, in CSS terms.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutStyle {
    /// Box type.
    pub display: Display,
    /// Flow or absolute placement.
    pub position: Position,
    /// Offsets for absolute placement: top, right, bottom, left.
    pub inset: [Dimension; 4],
    /// Preferred width.
    pub width: Dimension,
    /// Preferred height.
    pub height: Dimension,
    /// Minimum width.
    pub min_width: Dimension,
    /// Minimum height.
    pub min_height: Dimension,
    /// Maximum width.
    pub max_width: Dimension,
    /// Maximum height.
    pub max_height: Dimension,
    /// Width divided by height.
    pub aspect_ratio: Option<f32>,
    /// Inner spacing.
    pub padding: Insets,
    /// Outer spacing.
    pub margin: Insets,
    /// Gap between columns (horizontal) and rows (vertical).
    pub gap: (f32, f32),
    /// Main axis.
    pub flex_direction: FlexDirection,
    /// Whether items wrap onto new lines.
    pub flex_wrap: bool,
    /// Cross-axis alignment of children.
    pub align_items: Option<Align>,
    /// Cross-axis alignment of this node inside its parent.
    pub align_self: Option<Align>,
    /// Main-axis distribution of children.
    pub justify_content: Option<Justify>,
    /// Growth factor.
    pub flex_grow: f32,
    /// Shrink factor.
    pub flex_shrink: f32,
    /// Initial main size.
    pub flex_basis: Dimension,
    /// Grid column tracks.
    pub grid_columns: Vec<GridTrack>,
    /// Grid row tracks.
    pub grid_rows: Vec<GridTrack>,
    /// Columns this node spans in a grid.
    pub column_span: u16,
    /// Rows this node spans in a grid.
    pub row_span: u16,
    /// Overflow handling.
    pub overflow: Overflow,
}

impl Default for LayoutStyle {
    fn default() -> Self {
        Self {
            display: Display::Flex,
            position: Position::Relative,
            inset: [Dimension::Auto; 4],
            width: Dimension::Auto,
            height: Dimension::Auto,
            min_width: Dimension::Auto,
            min_height: Dimension::Auto,
            max_width: Dimension::Auto,
            max_height: Dimension::Auto,
            aspect_ratio: None,
            padding: Insets::ZERO,
            margin: Insets::ZERO,
            gap: (0.0, 0.0),
            flex_direction: FlexDirection::Column,
            flex_wrap: false,
            align_items: None,
            align_self: None,
            justify_content: None,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: Dimension::Auto,
            grid_columns: Vec::new(),
            grid_rows: Vec::new(),
            column_span: 1,
            row_span: 1,
            overflow: Overflow::Visible,
        }
    }
}
