use std::time::Duration;

use geometry::{Point, Vector};

use crate::{VELOCITY_SAMPLE_CAPACITY, VELOCITY_WINDOW};

#[derive(Debug, Clone, Copy, Default)]
struct Sample {
    timestamp: Duration,
    position: Point,
}

/// Estimates a pointer's velocity from its recent positions with a least-squares line fit.
///
/// Keeps the last [`VELOCITY_SAMPLE_CAPACITY`] samples in a fixed ring buffer and fits only those
/// within [`VELOCITY_WINDOW`] of the newest one, so a pointer that rested before release reports
/// zero velocity.
#[derive(Debug, Clone)]
pub struct VelocityTracker {
    samples: [Sample; VELOCITY_SAMPLE_CAPACITY],
    newest: usize,
    len: usize,
}

impl Default for VelocityTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl VelocityTracker {
    /// An empty tracker.
    pub fn new() -> Self {
        Self {
            samples: [Sample::default(); VELOCITY_SAMPLE_CAPACITY],
            newest: 0,
            len: 0,
        }
    }

    /// Records the pointer at `position` at monotonic time `timestamp`.
    pub fn add_sample(&mut self, timestamp: Duration, position: Point) {
        self.newest = (self.newest + 1) % VELOCITY_SAMPLE_CAPACITY;
        self.samples[self.newest] = Sample {
            timestamp,
            position,
        };
        self.len = (self.len + 1).min(VELOCITY_SAMPLE_CAPACITY);
    }

    /// Forgets every sample.
    pub fn reset(&mut self) {
        self.len = 0;
    }

    /// Velocity in logical pixels per second; zero with fewer than two recent samples.
    pub fn velocity(&self) -> Vector {
        if self.len < 2 {
            return Vector::ZERO;
        }
        let newest = self.samples[self.newest];
        let recent = (0..self.len)
            .map(|age| {
                self.samples
                    [(self.newest + VELOCITY_SAMPLE_CAPACITY - age) % VELOCITY_SAMPLE_CAPACITY]
            })
            .take_while(|sample| {
                newest.timestamp.saturating_sub(sample.timestamp) <= VELOCITY_WINDOW
            });

        let mut fit = LineFit::default();
        for sample in recent {
            let offset = sample.position - newest.position;
            fit.add(
                sample.timestamp.as_secs_f64() - newest.timestamp.as_secs_f64(),
                f64::from(offset.x),
                f64::from(offset.y),
            );
        }
        fit.slopes()
            .map_or(Vector::ZERO, |(x, y)| Vector::new(x as f32, y as f32))
    }
}

#[derive(Debug, Default)]
struct LineFit {
    count: f64,
    sum_t: f64,
    sum_tt: f64,
    sum_x: f64,
    sum_tx: f64,
    sum_y: f64,
    sum_ty: f64,
}

impl LineFit {
    fn add(&mut self, t: f64, x: f64, y: f64) {
        self.count += 1.0;
        self.sum_t += t;
        self.sum_tt += t * t;
        self.sum_x += x;
        self.sum_tx += t * x;
        self.sum_y += y;
        self.sum_ty += t * y;
    }

    fn slopes(&self) -> Option<(f64, f64)> {
        let time_spread = self.count * self.sum_tt - self.sum_t * self.sum_t;
        (time_spread > f64::EPSILON).then(|| {
            (
                (self.count * self.sum_tx - self.sum_t * self.sum_x) / time_spread,
                (self.count * self.sum_ty - self.sum_t * self.sum_y) / time_spread,
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_INTERVAL: Duration = Duration::from_millis(8);

    fn constant_velocity_tracker(velocity: Vector, samples: u32) -> VelocityTracker {
        let mut tracker = VelocityTracker::new();
        for index in 0..samples {
            let timestamp = SAMPLE_INTERVAL * index;
            let seconds = timestamp.as_secs_f32();
            tracker.add_sample(timestamp, Point::new(100.0, 50.0) + velocity * seconds);
        }
        tracker
    }

    fn within_two_percent(estimate: f32, truth: f32) -> bool {
        (estimate - truth).abs() <= truth.abs() * 0.02
    }

    #[test]
    fn constant_velocity_is_estimated_within_two_percent() {
        let truth = Vector::new(820.0, -340.0);
        let estimate = constant_velocity_tracker(truth, 60).velocity();
        assert!(within_two_percent(estimate.x, truth.x), "{estimate:?}");
        assert!(within_two_percent(estimate.y, truth.y), "{estimate:?}");
    }

    #[test]
    fn fewer_than_two_samples_give_zero() {
        let mut tracker = VelocityTracker::new();
        assert_eq!(tracker.velocity(), Vector::ZERO);
        tracker.add_sample(Duration::ZERO, Point::ZERO);
        assert_eq!(tracker.velocity(), Vector::ZERO);
    }

    #[test]
    fn samples_older_than_the_window_are_ignored() {
        let mut tracker = constant_velocity_tracker(Vector::new(500.0, 0.0), 10);
        tracker.add_sample(Duration::from_secs(1), Point::new(900.0, 50.0));
        assert_eq!(tracker.velocity(), Vector::ZERO);
    }

    #[test]
    fn reset_forgets_samples() {
        let mut tracker = constant_velocity_tracker(Vector::new(500.0, 0.0), 10);
        tracker.reset();
        assert_eq!(tracker.velocity(), Vector::ZERO);
    }
}
