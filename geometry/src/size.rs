/// A width and height in logical pixels.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Size {
    /// Horizontal extent.
    pub width: f32,
    /// Vertical extent.
    pub height: f32,
}

impl Size {
    /// An empty size.
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// Creates a size.
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Whether either side is zero or negative.
    pub fn is_empty(self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    /// Scales both sides by `factor`.
    pub fn scale(self, factor: f32) -> Self {
        Self::new(self.width * factor, self.height * factor)
    }
}
