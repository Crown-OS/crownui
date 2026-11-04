use super::skin::ButtonSize;

/// How a button rounds its corners.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonShape {
    /// The corner [`ButtonSize`] gives: a pill for text, a rounded square for an icon.
    #[default]
    Sized,
    /// Fully rounded ends at whatever height the button is laid out, a circle when square.
    Pill,
}

impl ButtonShape {
    pub(super) const fn radius(self, size: ButtonSize, height: f32) -> f32 {
        match self {
            Self::Sized => size.radius(),
            Self::Pill => height / 2.0,
        }
    }
}
