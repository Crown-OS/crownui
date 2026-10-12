use std::time::Duration;

/// A CSS `cubic-bezier(x1, y1, x2, y2)` easing function.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CubicBezier {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

/// A duration plus an easing function.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimingCurve {
    /// Total duration.
    pub duration: Duration,
    /// Easing applied to progress.
    pub easing: CubicBezier,
    /// Whether the animation starts over from its first value when it ends, forever.
    pub repeats: bool,
}

impl CubicBezier {
    /// CSS `linear`.
    pub const LINEAR: Self = Self::new(0.0, 0.0, 1.0, 1.0);
    /// CSS `ease`.
    pub const EASE: Self = Self::new(0.25, 0.1, 0.25, 1.0);
    /// CSS `ease-in`.
    pub const EASE_IN: Self = Self::new(0.42, 0.0, 1.0, 1.0);
    /// CSS `ease-out`.
    pub const EASE_OUT: Self = Self::new(0.0, 0.0, 0.58, 1.0);
    /// CSS `ease-in-out`.
    pub const EASE_IN_OUT: Self = Self::new(0.42, 0.0, 0.58, 1.0);

    /// Creates a curve from its two control points; `x` values are clamped to `0..=1`.
    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self {
            x1: x1.clamp(0.0, 1.0),
            y1,
            x2: x2.clamp(0.0, 1.0),
            y2,
        }
    }

    fn component(first: f32, second: f32, t: f32) -> f32 {
        let inverse = 1.0 - t;
        3.0 * inverse * inverse * t * first + 3.0 * inverse * t * t * second + t * t * t
    }

    fn component_slope(first: f32, second: f32, t: f32) -> f32 {
        let inverse = 1.0 - t;
        3.0 * inverse * inverse * first
            + 6.0 * inverse * t * (second - first)
            + 3.0 * t * t * (1.0 - second)
    }

    fn parameter_for(self, x: f32) -> f32 {
        let mut t = x;
        for _ in 0..8 {
            let error = Self::component(self.x1, self.x2, t) - x;
            let slope = Self::component_slope(self.x1, self.x2, t);
            if error.abs() < 1e-6 {
                return t;
            }
            if slope.abs() < 1e-6 {
                break;
            }
            t = (t - error / slope).clamp(0.0, 1.0);
        }
        let (mut low, mut high) = (0.0, 1.0);
        for _ in 0..24 {
            t = (low + high) / 2.0;
            if Self::component(self.x1, self.x2, t) < x {
                low = t;
            } else {
                high = t;
            }
        }
        t
    }

    /// Eased progress for linear progress `x` in `0..=1`.
    pub fn ease(self, x: f32) -> f32 {
        if x <= 0.0 {
            return 0.0;
        }
        if x >= 1.0 {
            return 1.0;
        }
        Self::component(self.y1, self.y2, self.parameter_for(x))
    }

    /// d(eased)/d(x) at linear progress `x`.
    pub fn slope(self, x: f32) -> f32 {
        let t = self.parameter_for(x.clamp(0.0, 1.0));
        let dx = Self::component_slope(self.x1, self.x2, t);
        if dx.abs() < 1e-6 {
            return 0.0;
        }
        Self::component_slope(self.y1, self.y2, t) / dx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints_are_fixed_and_linear_is_identity() {
        for curve in [
            CubicBezier::EASE,
            CubicBezier::EASE_IN_OUT,
            CubicBezier::EASE_OUT,
        ] {
            assert_eq!(curve.ease(0.0), 0.0);
            assert_eq!(curve.ease(1.0), 1.0);
        }
        assert!((CubicBezier::LINEAR.ease(0.37) - 0.37).abs() < 1e-4);
    }

    #[test]
    fn ease_in_out_is_symmetric() {
        let curve = CubicBezier::EASE_IN_OUT;
        assert!((curve.ease(0.3) + curve.ease(0.7) - 1.0).abs() < 1e-3);
    }
}
