use std::time::Duration;

use crate::{Animatable, Curve};

const SETTLE_DISPLACEMENT: f32 = 1e-3;
const SETTLE_VELOCITY: f32 = 1e-2;

/// One animation segment of a property.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Animation<T: Animatable> {
    from: T,
    to: T,
    initial_velocity: T,
    start: Duration,
    curve: Curve,
}

/// A sampled animation state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample<T> {
    /// Current value.
    pub value: T,
    /// Current velocity per second.
    pub velocity: T,
    /// Whether the animation reached its target and stopped moving.
    pub finished: bool,
}

impl<T: Animatable> Animation<T> {
    /// Animates from `from` to `to` starting at `start`, already moving at `initial_velocity`.
    pub fn new(from: T, to: T, initial_velocity: T, start: Duration, curve: Curve) -> Self {
        Self {
            from,
            to,
            initial_velocity,
            start,
            curve,
        }
    }

    /// The value this animation ends at.
    pub fn target(&self) -> T {
        self.to
    }

    /// The state at `now`.
    pub fn sample(&self, now: Duration) -> Sample<T> {
        let elapsed = now.saturating_sub(self.start).as_secs_f32();
        match self.curve {
            Curve::Spring(spring) => {
                let offset = self.from.sub(self.to);
                let coefficients = spring.coefficients(elapsed);
                let displacement = offset.scale(coefficients.position_from_offset).add(
                    self.initial_velocity
                        .scale(coefficients.position_from_velocity),
                );
                let velocity = offset.scale(coefficients.velocity_from_offset).add(
                    self.initial_velocity
                        .scale(coefficients.velocity_from_velocity),
                );
                let finished = displacement.magnitude() < SETTLE_DISPLACEMENT
                    && velocity.magnitude() < SETTLE_VELOCITY;
                Sample {
                    value: if finished {
                        self.to
                    } else {
                        self.to.add(displacement)
                    },
                    velocity: if finished {
                        self.to.sub(self.to)
                    } else {
                        velocity
                    },
                    finished,
                }
            }
            Curve::Timing(timing) => {
                let duration = timing.duration.as_secs_f32().max(1e-6);
                let cycles = elapsed / duration;
                let progress = if timing.repeats {
                    cycles.fract()
                } else {
                    cycles.min(1.0)
                };
                let finished = !timing.repeats && progress >= 1.0;
                let span = self.to.sub(self.from);
                Sample {
                    value: self.from.add(span.scale(timing.easing.ease(progress))),
                    velocity: if finished {
                        span.scale(0.0)
                    } else {
                        span.scale(timing.easing.slope(progress) / duration)
                    },
                    finished,
                }
            }
        }
    }

    /// A new segment toward `to` that starts from this one's value and velocity at `now`.
    pub fn retarget(&self, now: Duration, to: T, curve: Curve) -> Self {
        let current = self.sample(now);
        Self::new(current.value, to, current.velocity, now, curve)
    }
}

#[cfg(test)]
mod tests {
    use geometry::Point;

    use super::*;
    use crate::Spring;

    #[test]
    fn springs_reach_the_target_exactly_once_settled() {
        let animation =
            Animation::new(0.0_f32, 100.0, 0.0, Duration::ZERO, Spring::smooth().into());
        let sample = animation.sample(Duration::from_secs(3));
        assert!(sample.finished);
        assert_eq!(sample.value, 100.0);
    }

    #[test]
    fn retargeting_keeps_position_and_velocity_continuous() {
        let first = Animation::new(
            Point::ZERO,
            Point::new(100.0, 0.0),
            Point::ZERO,
            Duration::ZERO,
            Spring::snappy().into(),
        );
        let switch_at = Duration::from_millis(80);
        let before = first.sample(switch_at);
        let second = first.retarget(switch_at, Point::new(0.0, 100.0), Spring::snappy().into());
        let after = second.sample(switch_at);
        assert_eq!(before.value, after.value);
        assert_eq!(before.velocity, after.velocity);
    }

    #[test]
    fn repeating_timing_curves_loop_and_never_finish() {
        let animation = Animation::new(
            0.0_f32,
            360.0,
            0.0,
            Duration::ZERO,
            Curve::linear(1000).repeating(),
        );
        assert!((animation.sample(Duration::from_millis(250)).value - 90.0).abs() < 1e-3);
        let second_turn = animation.sample(Duration::from_millis(1250));
        assert!((second_turn.value - 90.0).abs() < 1e-2);
        assert!(!animation.sample(Duration::from_secs(3600)).finished);
    }

    #[test]
    fn timing_curves_finish_at_their_duration() {
        let animation =
            Animation::new(10.0_f32, 20.0, 0.0, Duration::ZERO, Curve::ease_in_out(200));
        assert!(!animation.sample(Duration::from_millis(199)).finished);
        let done = animation.sample(Duration::from_millis(200));
        assert!(done.finished);
        assert_eq!(done.value, 20.0);
    }
}
