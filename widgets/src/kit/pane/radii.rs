use geometry::CornerRadii;

/// Corner radii, clockwise from the top left.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Radii {
    /// Top left corner.
    pub top_left: f32,
    /// Top right corner.
    pub top_right: f32,
    /// Bottom right corner.
    pub bottom_right: f32,
    /// Bottom left corner.
    pub bottom_left: f32,
}

impl Radii {
    /// The same radius on all four corners.
    pub const fn all(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    /// Rounded down the left edge, square down the right: the left half of a split card.
    pub const fn left(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: 0.0,
            bottom_right: 0.0,
            bottom_left: radius,
        }
    }

    /// Rounded down the right edge, square down the left.
    pub const fn right(radius: f32) -> Self {
        Self {
            top_left: 0.0,
            top_right: radius,
            bottom_right: radius,
            bottom_left: 0.0,
        }
    }

    pub(super) fn inset(self, amount: f32) -> CornerRadii {
        let shrink = |radius: f32| (radius - amount).max(0.0);
        CornerRadii {
            top_left: shrink(self.top_left),
            top_right: shrink(self.top_right),
            bottom_right: shrink(self.bottom_right),
            bottom_left: shrink(self.bottom_left),
        }
    }
}

impl From<Radii> for CornerRadii {
    fn from(radii: Radii) -> Self {
        radii.inset(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn left_rounds_only_the_left_corners() {
        let radii = CornerRadii::from(Radii::left(24.0));
        assert_eq!((radii.top_left, radii.bottom_left), (24.0, 24.0));
        assert_eq!((radii.top_right, radii.bottom_right), (0.0, 0.0));
    }

    #[test]
    fn insetting_never_goes_negative() {
        let radii = Radii::right(0.25).inset(0.5);
        assert_eq!(radii, CornerRadii::ZERO);
    }
}
