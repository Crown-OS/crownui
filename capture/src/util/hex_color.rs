use geometry::Color;

/// `rrggbbaa`, the color format of scenes.
pub(crate) fn hex_color(color: Color) -> String {
    let [red, green, blue, alpha] = color.to_rgba8();
    format!("{red:02x}{green:02x}{blue:02x}{alpha:02x}")
}
