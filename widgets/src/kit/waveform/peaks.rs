use std::sync::Arc;

use view::{IntoProp, Prop};

use crate::util::unit_interval::{byte_to_unit, unit_to_byte};

/// The precomputed peaks of a saved recording, one `0..=255` byte per bucket, for a
/// [`waveform`](super::waveform). Shared, so cloning one into a signal copies a pointer; build it
/// once (off the UI thread for long takes) and the waveform fits it to whatever width it gets.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct WaveformPeaks(Arc<[u8]>);

impl WaveformPeaks {
    /// Peaks already packed into bytes, `255` being full scale.
    pub fn new(peaks: impl Into<Arc<[u8]>>) -> Self {
        Self(peaks.into())
    }

    /// Packs `0..=1` peaks into bytes; out-of-range values clamp.
    pub fn from_levels(levels: &[f32]) -> Self {
        Self(levels.iter().copied().map(unit_to_byte).collect())
    }

    /// The packed peaks.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// How many buckets there are.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether there are no buckets (an empty recording).
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The loudest bucket under bar `bar` of `bars` spread over the whole recording; with fewer
    /// buckets than bars, neighbouring bars share one.
    pub(super) fn bar_level(&self, bar: usize, bars: usize) -> f32 {
        let start = bar * self.0.len() / bars.max(1);
        let end = ((bar + 1) * self.0.len() / bars.max(1)).max(start + 1);
        self.0
            .get(start..end.min(self.0.len()))
            .and_then(|buckets| buckets.iter().max())
            .map_or(0.0, |&peak| byte_to_unit(peak))
    }
}

impl IntoProp<WaveformPeaks> for WaveformPeaks {
    fn into_prop(self) -> Prop<WaveformPeaks> {
        Prop::Static(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_bar_takes_the_loudest_of_its_buckets() {
        let peaks = WaveformPeaks::new(vec![10, 255, 0, 51]);
        assert_eq!(peaks.bar_level(0, 2), 1.0);
        assert_eq!(peaks.bar_level(1, 2), 0.2);
    }

    #[test]
    fn fewer_buckets_than_bars_stretch_and_no_buckets_are_silent() {
        let peaks = WaveformPeaks::from_levels(&[0.0, 1.0]);
        assert_eq!(peaks.bar_level(1, 4), 0.0);
        assert_eq!(peaks.bar_level(2, 4), 1.0);
        assert_eq!(WaveformPeaks::default().bar_level(3, 8), 0.0);
    }
}
