const INTER_FONT: &[u8] = include_bytes!("../../resources/fonts/Inter.ttf");

/// The family name the bundled Inter registers under.
pub const INTER_FAMILY: &str = "Inter Variable";

/// The kit's body text size in logical pixels.
pub const TEXT_SIZE: f32 = 14.0;

/// The bundled Inter, for `AppConfig::font(inter_font().to_vec(), Some(INTER_FAMILY))`.
pub const fn inter_font() -> &'static [u8] {
    INTER_FONT
}
