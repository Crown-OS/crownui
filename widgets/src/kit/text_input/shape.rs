use super::metrics::{FIELD_HEIGHT, FIELD_RADIUS};

/// How a field rounds its corners; chrome, ring and editor are identical in both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FieldShape {
    /// The kit's standard control corner.
    #[default]
    Rounded,
    /// Fully rounded ends, as a search bar wears.
    Pill,
}

impl FieldShape {
    pub(crate) const fn radius(self) -> f32 {
        match self {
            Self::Rounded => FIELD_RADIUS,
            Self::Pill => FIELD_HEIGHT / 2.0,
        }
    }
}
