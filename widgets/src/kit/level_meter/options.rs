use view::{Element, IntoProp};

use super::{axis::MeterAxis, kind::LevelMeterKind};

/// The options of a [`level_meter`](super::level_meter), chained before its modifiers.
pub trait LevelMeterOptions: Sized {
    /// The `0..=1` linear RMS of the same input, drawn solid over a dimmed peak.
    fn rms(self, rms: impl IntoProp<f32>) -> Self;

    /// Rises from the bottom instead of from the left: eight wide and 120 tall by default.
    fn vertical(self) -> Self;
}

impl LevelMeterOptions for Element<LevelMeterKind> {
    fn rms(self, rms: impl IntoProp<f32>) -> Self {
        self.configure(|kind| kind.rms = Some(rms.into_prop()))
    }

    fn vertical(self) -> Self {
        self.configure(|kind| kind.axis = MeterAxis::Vertical)
    }
}
