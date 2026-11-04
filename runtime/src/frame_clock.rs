use std::time::Duration;

use windowing::FrameTiming;

/// The times frames are sampled at: one display refresh apart while frame callbacks come back
/// to back, so motion advances by the same step each frame however late the thread wakes for it.
#[derive(Debug, Default)]
pub(crate) struct FrameClock {
    /// The time of the last frame that waited for its callback.
    paced: Option<Duration>,
    latest: Duration,
}

impl FrameClock {
    /// The time to sample the frame woken at `woke` with: the previous frame's time plus the
    /// refresh while that is within half a refresh of `woke`, otherwise `woke` itself. A frame
    /// that did not wait for its callback, as the first after a pause does, is not in step with
    /// the display, so it keeps its own time and does not set the pace. Times never go back.
    pub(crate) fn next(&mut self, woke: Duration, timing: FrameTiming) -> Duration {
        let predicted = self
            .paced
            .zip(timing.refresh)
            .filter(|_| timing.waited)
            .map(|(paced, refresh)| (paced + refresh, refresh));
        let time = match predicted {
            Some((predicted, refresh)) if predicted.abs_diff(woke) < refresh / 2 => predicted,
            _ => woke,
        }
        .max(self.latest);
        self.paced = timing.waited.then_some(time);
        self.latest = time;
        time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REFRESH: Duration = Duration::from_micros(6_944);

    fn ms(millis: f64) -> Duration {
        Duration::from_secs_f64(millis / 1000.0)
    }

    fn timing(waited: bool) -> FrameTiming {
        FrameTiming {
            refresh: Some(REFRESH),
            waited,
            ..FrameTiming::default()
        }
    }

    #[test]
    fn frames_woken_with_jitter_advance_by_exactly_one_refresh() {
        let mut clock = FrameClock::default();
        let first = clock.next(ms(100.0), timing(true));
        let second = clock.next(ms(100.0) + REFRESH + ms(1.5), timing(true));
        let third = clock.next(ms(100.0) + REFRESH * 2 - ms(2.0), timing(true));
        assert_eq!(first, ms(100.0));
        assert_eq!(second - first, REFRESH);
        assert_eq!(third - second, REFRESH);
    }

    #[test]
    fn a_missed_refresh_or_a_pause_starts_again_from_the_wake_time() {
        let mut clock = FrameClock::default();
        clock.next(ms(100.0), timing(true));
        let late = ms(100.0) + REFRESH * 2;
        assert_eq!(clock.next(late, timing(true)), late);
        assert_eq!(clock.next(ms(900.0), timing(true)), ms(900.0));
    }

    #[test]
    fn a_frame_that_did_not_wait_does_not_set_the_pace() {
        let mut clock = FrameClock::default();
        let off_grid = ms(103.0);
        assert_eq!(clock.next(off_grid, timing(false)), off_grid);
        let aligned = ms(107.0);
        assert_eq!(clock.next(aligned, timing(true)), aligned);
        assert_eq!(
            clock.next(aligned + REFRESH + ms(1.0), timing(true)),
            aligned + REFRESH
        );
        assert_eq!(
            clock.next(aligned + REFRESH - ms(0.5), timing(false)),
            aligned + REFRESH
        );
    }

    #[test]
    fn without_a_known_refresh_frames_use_their_wake_time_without_going_back() {
        let mut clock = FrameClock::default();
        let unpaced = FrameTiming {
            waited: true,
            ..FrameTiming::default()
        };
        clock.next(ms(100.0), unpaced);
        assert_eq!(clock.next(ms(107.3), unpaced), ms(107.3));
        assert_eq!(clock.next(ms(107.0), unpaced), ms(107.3));
    }
}
