use crate::kit::text_input::FIELD_HEIGHT;

pub(super) const CAP_HEIGHT: f32 = 22.0;
pub(super) const CAP_RADIUS: f32 = CAP_HEIGHT / 2.0;
pub(super) const CAP_PADDING: f32 = 8.0;
pub(super) const CAP_INSET: f32 = (FIELD_HEIGHT - CAP_HEIGHT) / 2.0;
pub(super) const FIELD_RADIUS: f32 = FIELD_HEIGHT / 2.0;
pub(super) const CAP_GAP: f32 = 5.0;
pub(super) const CAP_TEXT_SIZE: f32 = 12.0;
pub(super) const CAP_FILL_ALPHA: f32 = 0.10;
pub(super) const CAP_BORDER_ALPHA: f32 = 0.16;
pub(super) const CAP_GLOW_ALPHA: f32 = 0.08;
pub(super) const CAP_DISABLED_GLOW_ALPHA: f32 = 0.04;

pub(super) const RECORDING_HINT: &str = "Press a shortcut…";
pub(super) const PENDING_HINT: &str = "…";
pub(super) const UNBOUND_HINT: &str = "None";
