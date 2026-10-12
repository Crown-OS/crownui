use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

const NANOS_PER_SECOND: u64 = 1_000_000_000;

/// A lock-free token bucket (GCRA) allowing `rate` calls per second with a burst of `rate`.
#[derive(Debug)]
pub(crate) struct RateLimiter {
    rate: u32,
    interval_nanos: u64,
    burst_tolerance_nanos: u64,
    origin: Instant,
    theoretical_arrival_nanos: AtomicU64,
}

impl RateLimiter {
    pub(crate) fn per_second(rate: u32) -> Option<Self> {
        let interval_nanos = NANOS_PER_SECOND.checked_div(u64::from(rate))?;
        Some(Self {
            rate,
            interval_nanos,
            burst_tolerance_nanos: interval_nanos * u64::from(rate - 1),
            origin: Instant::now(),
            theoretical_arrival_nanos: AtomicU64::new(0),
        })
    }

    pub(crate) fn rate(&self) -> u32 {
        self.rate
    }

    pub(crate) fn try_acquire(&self) -> bool {
        let now = u64::try_from(self.origin.elapsed().as_nanos()).unwrap_or(u64::MAX);
        self.theoretical_arrival_nanos
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |arrival| {
                let start = arrival.max(now);
                (start - now <= self.burst_tolerance_nanos).then_some(start + self.interval_nanos)
            })
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::RateLimiter;

    #[test]
    fn bursts_up_to_the_rate_then_refuses() {
        let Some(limiter) = RateLimiter::per_second(3) else {
            panic!("non-zero rate");
        };
        assert_eq!(
            (0..4).map(|_| limiter.try_acquire()).collect::<Vec<_>>(),
            [true, true, true, false]
        );
        assert!(RateLimiter::per_second(0).is_none());
    }
}
