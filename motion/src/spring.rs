use std::f32::consts::TAU;

/// A damped harmonic oscillator described SwiftUI-style by its response and damping ratio.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spring {
    natural_frequency: f32,
    damping_ratio: f32,
}

/// How a unit offset and a unit velocity evolve after `t` seconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SpringCoefficients {
    pub(crate) position_from_offset: f32,
    pub(crate) position_from_velocity: f32,
    pub(crate) velocity_from_offset: f32,
    pub(crate) velocity_from_velocity: f32,
}

const CRITICAL_TOLERANCE: f32 = 1e-4;

impl Spring {
    /// `response` is the period of the undamped oscillation in seconds; `damping_ratio` 1 is
    /// critically damped, below 1 bounces.
    pub fn new(response: f32, damping_ratio: f32) -> Self {
        Self {
            natural_frequency: TAU / response.max(1e-3),
            damping_ratio: damping_ratio.max(0.0),
        }
    }

    /// Quick with a hint of overshoot; the default for state changes.
    pub fn snappy() -> Self {
        Self::new(0.25, 0.86)
    }

    /// Calm, critically damped; good for layout and theme changes.
    pub fn smooth() -> Self {
        Self::new(0.4, 1.0)
    }

    /// Very fast and critically damped; good for press feedback.
    pub fn crisp() -> Self {
        Self::new(0.15, 1.0)
    }

    /// Visibly bouncy.
    pub fn bouncy() -> Self {
        Self::new(0.5, 0.7)
    }

    /// Tracks a finger closely while dragging.
    pub fn interactive() -> Self {
        Self::new(0.15, 0.86)
    }

    /// Stiffness `k` for unit mass.
    pub fn stiffness(self) -> f32 {
        self.natural_frequency * self.natural_frequency
    }

    /// Damping `c` for unit mass.
    pub fn damping(self) -> f32 {
        2.0 * self.damping_ratio * self.natural_frequency
    }

    /// The damping ratio.
    pub fn damping_ratio(self) -> f32 {
        self.damping_ratio
    }

    pub(crate) fn coefficients(self, seconds: f32) -> SpringCoefficients {
        let omega = self.natural_frequency;
        let zeta = self.damping_ratio;
        let t = seconds.max(0.0);
        if (zeta - 1.0).abs() < CRITICAL_TOLERANCE {
            let decay = (-omega * t).exp();
            return SpringCoefficients {
                position_from_offset: (1.0 + omega * t) * decay,
                position_from_velocity: t * decay,
                velocity_from_offset: -omega * omega * t * decay,
                velocity_from_velocity: (1.0 - omega * t) * decay,
            };
        }
        if zeta < 1.0 {
            let decay_rate = zeta * omega;
            let damped = omega * (1.0 - zeta * zeta).sqrt();
            let decay = (-decay_rate * t).exp();
            let (sin, cos) = (damped * t).sin_cos();
            return SpringCoefficients {
                position_from_offset: decay * (cos + decay_rate / damped * sin),
                position_from_velocity: decay * sin / damped,
                velocity_from_offset: -decay * omega * omega / damped * sin,
                velocity_from_velocity: decay * (cos - decay_rate / damped * sin),
            };
        }
        let root = (zeta * zeta - 1.0).sqrt();
        let fast = -omega * (zeta + root);
        let slow = -omega * (zeta - root);
        let (fast_decay, slow_decay) = ((fast * t).exp(), (slow * t).exp());
        let span = slow - fast;
        SpringCoefficients {
            position_from_offset: (slow * fast_decay - fast * slow_decay) / span,
            position_from_velocity: (slow_decay - fast_decay) / span,
            velocity_from_offset: fast * slow * (fast_decay - slow_decay) / span,
            velocity_from_velocity: (slow * slow_decay - fast * fast_decay) / span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn position(spring: Spring, offset: f32, velocity: f32, seconds: f32) -> f32 {
        let coefficients = spring.coefficients(seconds);
        offset * coefficients.position_from_offset + velocity * coefficients.position_from_velocity
    }

    #[test]
    fn every_damping_regime_starts_at_the_initial_conditions() {
        for spring in [Spring::new(0.3, 0.5), Spring::new(0.3, 1.0), Spring::new(0.3, 2.0)] {
            let start = spring.coefficients(0.0);
            assert!((start.position_from_offset - 1.0).abs() < 1e-5);
            assert!(start.position_from_velocity.abs() < 1e-5);
            assert!((start.velocity_from_velocity - 1.0).abs() < 1e-5);
            assert!(start.velocity_from_offset.abs() < 1e-5);
        }
    }

    #[test]
    fn analytic_velocity_matches_the_numeric_derivative() {
        let step = 1e-3;
        for spring in [Spring::new(0.3, 0.4), Spring::new(0.3, 1.0), Spring::new(0.3, 1.7)] {
            for seconds in [0.05, 0.1, 0.2] {
                let numeric = (position(spring, 100.0, -50.0, seconds + step)
                    - position(spring, 100.0, -50.0, seconds - step))
                    / (2.0 * step);
                let coefficients = spring.coefficients(seconds);
                let analytic = 100.0 * coefficients.velocity_from_offset
                    - 50.0 * coefficients.velocity_from_velocity;
                assert!((numeric - analytic).abs() < 0.5, "{numeric} vs {analytic}");
            }
        }
    }

    #[test]
    fn springs_settle_within_a_few_responses() {
        assert!(position(Spring::smooth(), 100.0, 0.0, 1.2).abs() < 0.1);
        assert!(position(Spring::snappy(), 100.0, 0.0, 1.0).abs() < 0.1);
    }

    #[test]
    fn critical_damping_never_overshoots() {
        let overshoots = (1..100).any(|step| position(Spring::smooth(), 100.0, 0.0, step as f32 * 0.01) < -1e-3);
        assert!(!overshoots);
    }
}
