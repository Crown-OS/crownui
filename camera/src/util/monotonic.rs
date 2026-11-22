use media::ClockNanos;
use rustix::time::{ClockId, clock_gettime};

const NANOS_PER_SECOND: u64 = 1_000_000_000;

/// Now on `CLOCK_MONOTONIC`, which the PipeWire graph clock follows.
pub(crate) fn now() -> ClockNanos {
    let now = clock_gettime(ClockId::Monotonic);
    ClockNanos(now.tv_sec.unsigned_abs() * NANOS_PER_SECOND + now.tv_nsec.unsigned_abs())
}
