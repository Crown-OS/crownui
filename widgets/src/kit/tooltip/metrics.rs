use std::time::Duration;

pub(super) const DELAY: Duration = Duration::from_millis(500);
/// Closest a label comes to the window's left and right edges.
pub(super) const EDGE_MARGIN: f32 = 4.0;
/// Widest a label grows before it wraps.
pub(super) const MAX_WIDTH: f32 = 320.0;
pub(super) const GAP: f32 = 6.0;
pub(super) const RADIUS: f32 = 6.0;
pub(super) const H_PADDING: f32 = 8.0;
pub(super) const V_PADDING: f32 = 4.0;
pub(super) const TEXT_SIZE: f32 = 12.0;
pub(super) const LINE_HEIGHT: f32 = 1.2;
pub(super) const BORDER_WIDTH: f32 = 1.0;
pub(super) const SHADOW_BLUR: f32 = 12.0;
pub(super) const SHADOW_OFFSET_Y: f32 = 4.0;
pub(super) const APPEAR_SCALE: f32 = 0.96;

/// Height of a one-line bubble, for deciding whether it fits below its anchor.
pub(super) const BUBBLE_HEIGHT: f32 = TEXT_SIZE * LINE_HEIGHT + 2.0 * V_PADDING;
