use geometry::Color;

/// One run of a [`segmented_bar`](super::segmented_bar): how much of the bar it takes and its color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    /// Relative size; the bar divides by the total, so raw quantities work.
    pub weight: f32,
    /// The run's fill.
    pub color: Color,
}

impl Segment {
    /// A run of `weight` painted `color`.
    pub const fn new(weight: f32, color: Color) -> Self {
        Self { weight, color }
    }

    /// A negative or NaN weight contributes nothing instead of corrupting every other run.
    pub(super) fn usable_weight(&self) -> f32 {
        if self.weight.is_nan() || self.weight < 0.0 {
            0.0
        } else {
            self.weight
        }
    }
}

#[cfg(test)]
mod tests {
    use geometry::BLACK;

    use super::*;

    #[test]
    fn negative_and_nan_weights_count_for_nothing() {
        assert_eq!(Segment::new(-5.0, BLACK).usable_weight(), 0.0);
        assert_eq!(Segment::new(f32::NAN, BLACK).usable_weight(), 0.0);
        assert_eq!(Segment::new(3.0, BLACK).usable_weight(), 3.0);
    }
}
