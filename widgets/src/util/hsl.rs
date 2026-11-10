use geometry::Color;

/// `color` with its HSL lightness and saturation moved by `lightness` and `saturation`, its hue
/// and alpha kept: a lighter or deeper shade of the same hue.
pub(crate) fn shift_hsl(color: Color, lightness: f32, saturation: f32) -> Color {
    let (hue, old_saturation, old_lightness) = to_hsl(color);
    let (r, g, b) = from_hsl(
        hue,
        (old_saturation + saturation).clamp(0.0, 1.0),
        (old_lightness + lightness).clamp(0.0, 1.0),
    );
    Color::new(r, g, b, color.a)
}

fn to_hsl(color: Color) -> (f32, f32, f32) {
    let (r, g, b) = (color.r, color.g, color.b);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let lightness = (max + min) / 2.0;
    let chroma = max - min;
    if chroma <= f32::EPSILON {
        return (0.0, 0.0, lightness);
    }
    let saturation = chroma / (1.0 - (2.0 * lightness - 1.0).abs());
    let sector = if max == r {
        ((g - b) / chroma).rem_euclid(6.0)
    } else if max == g {
        (b - r) / chroma + 2.0
    } else {
        (r - g) / chroma + 4.0
    };
    (sector / 6.0, saturation, lightness)
}

fn from_hsl(hue: f32, saturation: f32, lightness: f32) -> (f32, f32, f32) {
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let sector = hue * 6.0;
    let second = chroma * (1.0 - (sector.rem_euclid(2.0) - 1.0).abs());
    let (r, g, b) = match sector as u8 {
        0 => (chroma, second, 0.0),
        1 => (second, chroma, 0.0),
        2 => (0.0, chroma, second),
        3 => (0.0, second, chroma),
        4 => (second, 0.0, chroma),
        _ => (chroma, 0.0, second),
    };
    let base = lightness - chroma / 2.0;
    (r + base, g + base, b + base)
}

#[cfg(test)]
mod tests {
    use geometry::rgb;

    use super::*;

    #[test]
    fn no_shift_round_trips_the_color() {
        for hex in [0x6d48e8, 0x2563eb, 0x16a34a, 0xea580c, 0xdb2777, 0x808080] {
            assert_eq!(
                shift_hsl(rgb(hex), 0.0, 0.0).to_rgba8(),
                rgb(hex).to_rgba8()
            );
        }
    }

    #[test]
    fn shifts_keep_the_hue_and_alpha_and_stay_in_gamut() {
        let shifted = shift_hsl(rgb(0xff0000).with_alpha_factor(0.5), 0.9, 0.9);
        assert!((shifted.a - 0.5).abs() < 1e-6);
        for channel in [shifted.r, shifted.g, shifted.b] {
            assert!((0.0..=1.0).contains(&channel));
        }
        assert!(shifted.r >= shifted.g && (shifted.g - shifted.b).abs() < 1e-6);
    }
}
