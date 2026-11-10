use geometry::Vector;

pub(super) const TRACK_HEIGHT: f32 = 8.0;
pub(super) const THUMB_WIDTH: f32 = 26.0;
pub(super) const THUMB_HEIGHT: f32 = 20.0;
pub(super) const THUMB_HALF_WIDTH: f32 = THUMB_WIDTH / 2.0;
pub(super) const HEIGHT: f32 = THUMB_HEIGHT + 10.0;
pub(super) const MIN_WIDTH: f32 = 120.0;
pub(super) const MAX_WIDTH: f32 = 480.0;
pub(super) const THUMB_FILL_ALPHA: f32 = 0.85;
/// Blur of the track seen through the thumb (CSS `backdrop-filter`).
pub(super) const THUMB_BACKDROP_BLUR: f32 = 2.9;
pub(super) const THUMB_SHADOW_ALPHA: f32 = 0.07;
pub(super) const THUMB_SHADOW_OFFSET: Vector = Vector::new(1.0, 1.0);
pub(super) const THUMB_SHADOW_BLUR: f32 = 2.2;
pub(super) const WHEEL_NOTCH_SENSITIVITY: f32 = 3.5;
pub(super) const WHEEL_PIXEL_SENSITIVITY: f32 = 0.3;
pub(super) const SHIFT_MULTIPLIER: f32 = 10.0;
pub(super) const PAGE_MULTIPLIER: f32 = 10.0;
pub(super) const DEFAULT_STEPS: f32 = 100.0;
