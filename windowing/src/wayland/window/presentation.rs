use std::time::Duration;

use sctk::reexports::{
    client::{Connection, Dispatch, QueueHandle},
    protocols::wp::presentation_time::client::wp_presentation_feedback::{
        self, WpPresentationFeedback,
    },
};

use super::render_surface::FramePacer;
use crate::wayland::util::u64_halves::join_u64;

/// The presentation time and refresh interval of a `presented` event; a zero refresh means
/// the display has no fixed rate.
pub(crate) fn presentation_timing(
    tv_sec_hi: u32,
    tv_sec_lo: u32,
    tv_nsec: u32,
    refresh_nsec: u32,
) -> (Duration, Option<Duration>) {
    let presented_at = Duration::from_secs(join_u64(tv_sec_hi, tv_sec_lo))
        .saturating_add(Duration::from_nanos(u64::from(tv_nsec)));
    let refresh = (refresh_nsec != 0).then(|| Duration::from_nanos(u64::from(refresh_nsec)));
    (presented_at, refresh)
}

impl Dispatch<WpPresentationFeedback, ()> for FramePacer {
    fn event(
        pacer: &mut Self,
        _: &WpPresentationFeedback,
        event: wp_presentation_feedback::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wp_presentation_feedback::Event::Presented {
            tv_sec_hi,
            tv_sec_lo,
            tv_nsec,
            refresh,
            ..
        } = event
        {
            let (presented_at, refresh) =
                presentation_timing(tv_sec_hi, tv_sec_lo, tv_nsec, refresh);
            pacer.presented_at = Some(presented_at);
            pacer.refresh = refresh;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presented_events_become_durations() {
        let (presented_at, refresh) = presentation_timing(1, 2, 500, 16_666_667);
        assert_eq!(presented_at, Duration::new((1 << 32) | 2, 500));
        assert_eq!(refresh, Some(Duration::from_nanos(16_666_667)));
        assert_eq!(presentation_timing(0, 1, 0, 0).1, None);
    }
}
