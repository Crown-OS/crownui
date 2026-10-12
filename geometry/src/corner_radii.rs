/// Per-corner radii of a rounded rectangle, in logical pixels.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CornerRadii {
    /// Top-left corner.
    pub top_left: f32,
    /// Top-right corner.
    pub top_right: f32,
    /// Bottom-right corner.
    pub bottom_right: f32,
    /// Bottom-left corner.
    pub bottom_left: f32,
}

impl CornerRadii {
    /// Square corners.
    pub const ZERO: Self = Self::uniform(0.0);

    /// The same radius on every corner.
    pub const fn uniform(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    /// Whether every corner is square.
    pub fn is_zero(self) -> bool {
        self == Self::ZERO
    }

    /// The largest radius, useful for compositor shapes that take a single radius.
    pub fn max(self) -> f32 {
        self.top_left
            .max(self.top_right)
            .max(self.bottom_right)
            .max(self.bottom_left)
    }
}
