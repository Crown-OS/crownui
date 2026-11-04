use crate::kit::TEXT_SIZE;

/// A text field's height, so the group lines up with the search bar beside it.
pub(super) const HEIGHT: f32 = 34.0;
pub(super) const TRACK_PADDING: f32 = 3.0;
pub(super) const TRACK_BORDER_WIDTH: f32 = 1.0;
pub(super) const SEGMENT_PADDING: f32 = 12.0;
pub(super) const SEGMENT_HEIGHT: f32 = HEIGHT - 2.0 * TRACK_PADDING;
pub(super) const ICON_SIZE: f32 = 15.0;
pub(super) const ICON_STROKE: f32 = 1.8;
pub(super) const LABEL_SIZE: f32 = TEXT_SIZE - 1.0;
pub(super) const HOVER_WASH: f32 = 0.6;
