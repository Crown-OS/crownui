pub(super) const TRIGGER_WIDTH: f32 = 252.0;
pub(super) const TRIGGER_HEIGHT: f32 = 34.0;
pub(super) const TRIGGER_RADIUS: f32 = 16.0;
pub(super) const TRIGGER_H_PADDING: f32 = 10.0;
pub(super) const CHEVRON_BOX: f32 = 20.0;
pub(super) const CHEVRON_INSET: f32 = 8.0;
pub(super) const ICON_STROKE: f32 = 1.8;
pub(super) const VALUE_TEXT_WIDTH: f32 =
    TRIGGER_WIDTH - TRIGGER_H_PADDING - CHEVRON_INSET - CHEVRON_BOX;
pub(super) const BORDER_WIDTH: f32 = 1.0;
pub(super) const PANEL_RADIUS: f32 = 18.0;
pub(super) const PANEL_PADDING: f32 = 5.0;
pub(super) const ROW_HEIGHT: f32 = 30.0;
pub(super) const ROW_H_PADDING: f32 = 8.0;
pub(super) const ROW_RADIUS: f32 = 16.0;
pub(super) const CHECK_BOX: f32 = 16.0;
pub(super) const ROW_TEXT_WIDTH: f32 =
    TRIGGER_WIDTH - PANEL_PADDING * 2.0 - ROW_H_PADDING * 2.0 - CHECK_BOX;
pub(super) const VALUE_FADE_MILLIS: u64 = 300;

pub(super) const fn panel_height(option_count: usize) -> f32 {
    PANEL_PADDING * 2.0 + option_count as f32 * ROW_HEIGHT
}
