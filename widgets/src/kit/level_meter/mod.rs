mod axis;
mod kind;
mod metrics;
mod options;
mod paint;
mod reading;
mod zone;

pub use kind::LevelMeterKind;
pub use options::LevelMeterOptions;

use view::{Element, IntoProp, Styled};

use crate::util::theme_fade::theme_fade;

use axis::MeterAxis;
use metrics::THICKNESS;

/// A live input meter for a `0..=1` linear `peak`, fed once per update (about 60 Hz). It reads on
/// a 60 dB scale, turns the warning color from -6 dBFS and the danger color from -1 dBFS, and
/// holds the loudest peak for 60 updates before letting it fall. Horizontal and 240 by 8 by
/// default; see [`LevelMeterOptions`] for RMS and a vertical meter.
pub fn level_meter(peak: impl IntoProp<f32>) -> Element<LevelMeterKind> {
    Element::new(LevelMeterKind {
        peak: peak.into_prop(),
        rms: None,
        axis: MeterAxis::Horizontal,
    })
    .rounded(THICKNESS / 2.0)
    .transition(theme_fade())
}
