mod bar;
mod kind;
mod layout;
mod level_history;
mod live_kind;
mod metrics;
mod options;
mod peaks;
mod seek;

pub use kind::WaveformKind;
pub use level_history::LevelHistory;
pub use live_kind::LiveWaveformKind;
pub use options::WaveformOptions;
pub use peaks::WaveformPeaks;

use reactive::Signal;
use view::{Element, IntoProp};

/// A saved recording's `peaks` as rounded bars mirrored around the center line, fitted to the
/// width, with a playhead at `progress` (`0..=1`). Bars behind the playhead wear the accent, the
/// rest the muted text color, each brighter the louder it is. Make it seekable with
/// [`WaveformOptions::on_seeking`].
pub fn waveform(
    peaks: impl IntoProp<WaveformPeaks>,
    progress: impl IntoProp<f32>,
) -> Element<WaveformKind> {
    Element::new(WaveformKind {
        peaks: peaks.into_prop(),
        progress: progress.into_prop(),
        on_seeking: None,
    })
}

/// The input while recording: the newest level of `history` at the right edge and older ones
/// scrolling left, one bar per push, repainting only when `history` changes.
pub fn live_waveform(history: Signal<LevelHistory>) -> Element<LiveWaveformKind> {
    Element::new(LiveWaveformKind { history })
}
