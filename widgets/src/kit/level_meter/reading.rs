use super::metrics::{FLOOR_DECIBELS, HOLD_FALL_PER_UPDATE, HOLD_UPDATES};
use crate::util::unit_interval::clamp_unit;

/// A linear `0..=1` amplitude on the meter's decibel scale: the floor at `0`, full scale at `1`.
pub(super) fn position_of(linear: f32) -> f32 {
    let decibels = (20.0 * clamp_unit(linear).log10()).max(FLOOR_DECIBELS);
    1.0 - decibels / FLOOR_DECIBELS
}

pub(super) fn decibels_at(position: f32) -> f32 {
    (1.0 - position) * FLOOR_DECIBELS
}

/// What the meter shows, in scale positions; the hold is clocked by the updates it is fed.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) struct MeterReading {
    pub(super) peak: f32,
    pub(super) rms: Option<f32>,
    pub(super) hold: f32,
    updates_held: u32,
}

impl MeterReading {
    pub(super) fn follow(&mut self, peak: f32, rms: Option<f32>) {
        self.peak = position_of(peak);
        self.rms = rms.map(position_of);
        if self.peak >= self.hold {
            self.hold = self.peak;
            self.updates_held = 0;
            return;
        }
        self.updates_held = self.updates_held.saturating_add(1);
        if self.updates_held > HOLD_UPDATES {
            self.hold = (self.hold - HOLD_FALL_PER_UPDATE).max(self.peak);
        }
    }

    pub(super) fn peak_decibels(&self) -> f32 {
        decibels_at(self.peak)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scale_runs_from_the_sixty_decibel_floor_to_full_scale() {
        assert_eq!(position_of(1.0), 1.0);
        assert_eq!(position_of(0.0), 0.0);
        assert_eq!(position_of(f32::NAN), 0.0);
        assert!((position_of(0.1) - 2.0 / 3.0).abs() < 1e-5);
        assert!((decibels_at(position_of(0.5)) + 6.02).abs() < 0.01);
    }

    #[test]
    fn the_hold_waits_a_second_of_updates_then_falls_to_the_peak() {
        let mut reading = MeterReading::default();
        reading.follow(1.0, None);
        for _ in 0..HOLD_UPDATES {
            reading.follow(0.1, None);
        }
        assert_eq!(reading.hold, 1.0);
        reading.follow(0.1, None);
        assert!(reading.hold < 1.0);
        for _ in 0..100 {
            reading.follow(0.1, None);
        }
        assert_eq!(reading.hold, reading.peak);
    }

    #[test]
    fn a_louder_peak_lifts_the_hold_at_once() {
        let mut reading = MeterReading::default();
        reading.follow(0.1, Some(0.05));
        reading.follow(0.5, Some(0.2));
        assert_eq!(reading.hold, reading.peak);
        assert!(reading.rms.is_some_and(|rms| rms < reading.peak));
    }
}
