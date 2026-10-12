use std::time::Duration;

use rustix::time::{ClockId, clock_gettime};

/// The CLOCK_MONOTONIC instant the compositor times animations and
/// presentation against.
pub fn monotonic_now() -> Duration {
    let now = clock_gettime(ClockId::Monotonic);
    Duration::new(
        u64::try_from(now.tv_sec).unwrap_or(0),
        u32::try_from(now.tv_nsec).unwrap_or(0),
    )
}
