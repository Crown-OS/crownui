use geometry::{Color, CornerRadii, Point, Rect, Size};
use graphics::DrawList;

use super::metrics::{BAR_GAP, BAR_PITCH, BAR_WIDTH, QUIET_BRIGHTNESS};
use crate::util::color::scale_alpha;

/// How many bars fit across `width`.
pub(super) fn bar_count(width: f32) -> usize {
    ((width + BAR_GAP) / BAR_PITCH).max(0.0) as usize
}

/// A capsule mirrored around the center line at `x`, a dot at silence and the full height at
/// full scale, brighter the louder it is.
pub(super) fn paint_bar(list: &mut DrawList, x: f32, level: f32, size: Size, color: Color) {
    let height = BAR_WIDTH + (size.height - BAR_WIDTH).max(0.0) * level;
    list.fill(
        Rect::new(
            Point::new(x, (size.height - height) / 2.0),
            Size::new(BAR_WIDTH, height),
        ),
        CornerRadii::uniform(BAR_WIDTH / 2.0),
        scale_alpha(color, QUIET_BRIGHTNESS + (1.0 - QUIET_BRIGHTNESS) * level),
    );
}
