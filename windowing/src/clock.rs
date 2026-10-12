use std::time::{Duration, Instant};

/// A monotonic clock; injected so tests and previews can step time deterministically.
pub trait Clock {
    /// Time elapsed since the clock's own epoch.
    fn now(&self) -> Duration;
}

/// The real monotonic clock.
#[derive(Debug, Clone, Copy)]
pub struct SystemClock {
    epoch: Instant,
}

impl Default for SystemClock {
    fn default() -> Self {
        Self {
            epoch: Instant::now(),
        }
    }
}

impl Clock for SystemClock {
    fn now(&self) -> Duration {
        self.epoch.elapsed()
    }
}

/// A clock that only moves when told to.
#[derive(Debug, Clone, Copy, Default)]
pub struct ManualClock {
    now: Duration,
}

impl ManualClock {
    /// Moves time forward.
    pub fn advance(&mut self, elapsed: Duration) {
        self.now += elapsed;
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Duration {
        self.now
    }
}
