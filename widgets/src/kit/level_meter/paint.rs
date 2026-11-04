use geometry::{Color, CornerRadii, Size};
use graphics::DrawList;

use super::{
    axis::MeterAxis,
    metrics::{HOLD_RADIUS, HOLD_THICKNESS, PEAK_BEHIND_RMS_ALPHA},
    reading::MeterReading,
    zone::MeterZone,
};
use crate::{kit::Theme, util::color::scale_alpha};

/// Draws the peak (dimmed behind the RMS when there is one) as a capsule from the quiet end, and
/// the hold as a short tick wherever it has outlived the peak.
pub(super) fn paint_meter(
    reading: MeterReading,
    theme: &Theme,
    axis: MeterAxis,
    size: Size,
    list: &mut DrawList,
) {
    let (length, thickness) = (axis.length(size), axis.thickness(size));
    let radii = CornerRadii::uniform(thickness / 2.0);
    let level = |list: &mut DrawList, position: f32, color: Color| {
        if position > 0.0 {
            let reach = (position * length).max(thickness);
            list.fill(axis.stretch(size, 0.0, reach), radii, color);
        }
    };
    let color = MeterZone::at(reading.peak).color(theme);
    match reading.rms {
        Some(rms) => {
            level(
                list,
                reading.peak,
                scale_alpha(color, PEAK_BEHIND_RMS_ALPHA),
            );
            level(list, rms, color);
        }
        None => level(list, reading.peak, color),
    }
    if reading.hold > reading.peak {
        let center = (reading.hold * length)
            .min(length - HOLD_THICKNESS / 2.0)
            .max(HOLD_THICKNESS / 2.0);
        list.fill(
            axis.stretch(
                size,
                center - HOLD_THICKNESS / 2.0,
                center + HOLD_THICKNESS / 2.0,
            ),
            CornerRadii::uniform(HOLD_RADIUS),
            MeterZone::at(reading.hold).color(theme),
        );
    }
}
