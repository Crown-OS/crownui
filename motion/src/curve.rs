use std::time::Duration;

use crate::{CubicBezier, Spring, TimingCurve};

/// How a property moves to a new value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Curve {
    /// Physically based; duration follows from the spring.
    Spring(Spring),
    /// Fixed duration with an easing function.
    Timing(TimingCurve),
}

impl Curve {
    /// CSS `linear` over `millis`.
    pub fn linear(millis: u64) -> Self {
        Self::timing(millis, CubicBezier::LINEAR)
    }

    /// CSS `ease` over `millis`.
    pub fn ease(millis: u64) -> Self {
        Self::timing(millis, CubicBezier::EASE)
    }

    /// CSS `ease-in` over `millis`.
    pub fn ease_in(millis: u64) -> Self {
        Self::timing(millis, CubicBezier::EASE_IN)
    }

    /// CSS `ease-out` over `millis`.
    pub fn ease_out(millis: u64) -> Self {
        Self::timing(millis, CubicBezier::EASE_OUT)
    }

    /// CSS `ease-in-out` over `millis`.
    pub fn ease_in_out(millis: u64) -> Self {
        Self::timing(millis, CubicBezier::EASE_IN_OUT)
    }

    fn timing(millis: u64, easing: CubicBezier) -> Self {
        Self::Timing(TimingCurve {
            duration: Duration::from_millis(millis),
            easing,
            repeats: false,
        })
    }

    /// This curve played over and over, for indicators that have no state to arrive at; springs,
    /// which settle by nature, are returned unchanged.
    #[must_use]
    pub fn repeating(self) -> Self {
        match self {
            Self::Timing(timing) => Self::Timing(TimingCurve {
                repeats: true,
                ..timing
            }),
            spring @ Self::Spring(_) => spring,
        }
    }
}

impl From<Spring> for Curve {
    fn from(spring: Spring) -> Self {
        Self::Spring(spring)
    }
}

/// The curve attached to a batch of property changes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transaction {
    /// How changed properties move.
    pub curve: Curve,
}

impl Transaction {
    /// A transaction animating with `curve`.
    pub fn new(curve: impl Into<Curve>) -> Self {
        Self {
            curve: curve.into(),
        }
    }
}
