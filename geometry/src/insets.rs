/// Distances inwards from each edge, as in CSS padding.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Insets {
    /// Top edge.
    pub top: f32,
    /// Right edge.
    pub right: f32,
    /// Bottom edge.
    pub bottom: f32,
    /// Left edge.
    pub left: f32,
}

impl Insets {
    /// No inset.
    pub const ZERO: Self = Self::uniform(0.0);

    /// The same inset on every edge.
    pub const fn uniform(length: f32) -> Self {
        Self {
            top: length,
            right: length,
            bottom: length,
            left: length,
        }
    }

    /// `horizontal` on left and right, `vertical` on top and bottom.
    pub const fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Left plus right.
    pub fn horizontal(self) -> f32 {
        self.left + self.right
    }

    /// Top plus bottom.
    pub fn vertical(self) -> f32 {
        self.top + self.bottom
    }
}
