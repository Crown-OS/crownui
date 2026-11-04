use crate::SAMPLE_RATE;

const NANOS_PER_SECOND: u64 = 1_000_000_000;

/// A time on the PipeWire graph clock (`CLOCK_MONOTONIC`) in nanoseconds.
///
/// Audio frames and PipeWire video frames carry the same clock, so a recorder lines them up
/// without translating timestamps.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClockNanos(pub u64);

impl ClockNanos {
    pub(crate) const fn after_samples(self, samples: usize) -> Self {
        Self(self.0 + samples as u64 * NANOS_PER_SECOND / SAMPLE_RATE as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_frame_of_samples_lasts_ten_milliseconds() {
        assert_eq!(ClockNanos(5).after_samples(480), ClockNanos(10_000_005));
    }
}
