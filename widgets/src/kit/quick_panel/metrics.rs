/// Every quick panel is this wide, whatever rows it holds.
pub const PANEL_WIDTH: f32 = 296.0;
/// Corner radius of the panel body, concentric with the capsule rows inset inside it.
pub const PANEL_RADIUS: f32 = ROW_INSET + ROW_RADIUS;

pub(super) const PAD_X: f32 = 16.0;
pub(super) const PAD_Y: f32 = 8.0;
pub(super) const ROW_INSET: f32 = 8.0;
pub(super) const ROW_RADIUS: f32 = ITEM_HEIGHT / 2.0;

pub(super) const HEADER_HEIGHT: f32 = 36.0;
pub(super) const SLIDER_HEIGHT: f32 = 40.0;
pub(super) const SECTION_HEIGHT: f32 = 28.0;
pub(super) const SECTION_DROP: f32 = 4.0;
pub(super) const ITEM_HEIGHT: f32 = 40.0;
pub(super) const ACTION_HEIGHT: f32 = 32.0;
pub(super) const SEPARATOR_HEIGHT: f32 = 16.0;
pub(super) const READOUT_HEIGHT: f32 = 64.0;
pub(super) const READOUT_PRIMARY_HEIGHT: f32 = 40.0;
pub(super) const READOUT_SECONDARY_HEIGHT: f32 = 20.0;

pub(super) const ICON_COLUMN: f32 = 28.0;
/// The icon circle sits as far inside its row as makes it concentric with the row's capsule end.
pub(super) const ICON_INSET: f32 = ROW_RADIUS - ICON_COLUMN / 2.0;
/// Where every label, and a slider's track, starts from the panel's edge.
pub(super) const LABEL_X: f32 = 56.0;
pub(super) const ICON_GAP: f32 = LABEL_X - ROW_INSET - ICON_INSET - ICON_COLUMN;
pub(super) const BADGE_GLYPH: f32 = 18.0;
pub(super) const PLAIN_GLYPH: f32 = 18.0;
pub(super) const CHEVRON: f32 = 14.0;
pub(super) const TRAILING_GAP: f32 = 8.0;
/// The slider's thumb overhangs its track by half a thumb, so the track starts where labels do.
pub(super) const SLIDER_OVERHANG: f32 = 13.0;

pub(super) const LINE_HEIGHT: f32 = 1.2;
pub(super) const DISPLAY_TEXT: (f32, u16) = (32.0, 500);
pub(super) const TITLE_TEXT: (f32, u16) = (15.0, 600);
pub(super) const BODY_TEXT: (f32, u16) = (13.0, 500);
pub(super) const DETAIL_TEXT: (f32, u16) = (13.0, 500);
pub(super) const CAPTION_TEXT: (f32, u16) = (12.0, 600);

pub(super) const PANEL_OPACITY: f32 = 0.66;
pub(super) const RIM_MIX: f32 = 0.16;
pub(super) const RIM_ALPHA: f32 = 0.55;

pub(super) const CHEVRON_RIGHT: &str = r#"<path d="M9.5 5.5 L16 12 L9.5 18.5"/>"#;
