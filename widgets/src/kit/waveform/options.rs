use view::{Cx, Element};

use super::kind::WaveformKind;

/// The options of a [`waveform`](super::waveform), chained before its listeners and modifiers.
pub trait WaveformOptions: Sized {
    /// Makes the waveform a seek bar, like the DOM media `seeking` event: `handler` gets the `0..=1`
    /// position under a click or drag, or one nudged by the arrows (5%), Home and End. Seek
    /// playback there and the `progress` signal brings the playhead along.
    fn on_seeking(self, handler: impl FnMut(&mut Cx, f32) + 'static) -> Self;
}

impl WaveformOptions for Element<WaveformKind> {
    fn on_seeking(self, handler: impl FnMut(&mut Cx, f32) + 'static) -> Self {
        self.configure(|kind| kind.on_seeking = Some(Box::new(handler)))
    }
}
