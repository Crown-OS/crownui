pub(super) const INSET_X: f32 = 16.0;
pub(super) const TITLE_HEIGHT: f32 = 36.0;
pub(super) const WEEKDAYS_HEIGHT: f32 = 24.0;
pub(super) const WEEK_HEIGHT: f32 = 32.0;
pub(super) const TODAY_DIAMETER: f32 = 28.0;
pub(super) const ARROW_SIZE: f32 = 14.0;
pub(super) const ARROW_TARGET: f32 = 32.0;
pub(super) const TITLE_TEXT: (f32, u16) = (15.0, 600);
pub(super) const CAPTION_TEXT: (f32, u16) = (12.0, 600);
pub(super) const DAY_TEXT: (f32, u16) = (13.0, 400);
pub(super) const LINE_HEIGHT: f32 = 1.2;
/// How much of the muted ink a day outside the shown month keeps.
pub(super) const OUTSIDE_MONTH_ALPHA: f32 = 0.45;
/// The same over dark glass, where faint is 0.36 against muted 0.66.
pub(super) const DARK_OUTSIDE_MONTH_ALPHA: f32 = 0.545;
pub(super) const WEEKDAY_INITIALS: [&str; 7] = ["M", "T", "W", "T", "F", "S", "S"];
pub(super) const CHEVRON_LEFT: &str = r#"<path d="M14.5 5.5 L8 12 L14.5 18.5"/>"#;
pub(super) const CHEVRON_RIGHT: &str = r#"<path d="M9.5 5.5 L16 12 L9.5 18.5"/>"#;
