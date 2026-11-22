/// Copies straight-alpha RGBA8 pixels into the premultiplied form Impeller samples.
pub(crate) fn premultiplied_rgba(straight: &[u8]) -> Vec<u8> {
    let multiply =
        |channel: u8, alpha: u8| ((u16::from(channel) * u16::from(alpha) + 127) / 255) as u8;
    let (pixels, _) = straight.as_chunks::<4>();
    pixels
        .iter()
        .flat_map(|&[red, green, blue, alpha]| {
            [
                multiply(red, alpha),
                multiply(green, alpha),
                multiply(blue, alpha),
                alpha,
            ]
        })
        .collect()
}

/// Turns a GL readback (bottom row first, premultiplied) into top-down straight-alpha RGBA8.
pub(crate) fn straight_top_down(width: u32, bottom_up_premultiplied: &[u8]) -> Vec<u8> {
    let divide = |channel: u8, alpha: u8| {
        ((u16::from(channel) * 255 + u16::from(alpha) / 2) / u16::from(alpha)).min(255) as u8
    };
    bottom_up_premultiplied
        .chunks_exact(width.max(1) as usize * 4)
        .rev()
        .flat_map(|row| row.as_chunks::<4>().0)
        .flat_map(|&[red, green, blue, alpha]| match alpha {
            0 => [0; 4],
            alpha => [
                divide(red, alpha),
                divide(green, alpha),
                divide(blue, alpha),
                alpha,
            ],
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_channels_are_scaled_by_alpha() {
        assert_eq!(
            premultiplied_rgba(&[255, 128, 0, 255, 255, 255, 255, 128, 10, 20, 30, 0]),
            [255, 128, 0, 255, 128, 128, 128, 128, 0, 0, 0, 0]
        );
    }

    #[test]
    fn readbacks_are_flipped_and_unpremultiplied() {
        let bottom_up = [
            10, 20, 30, 255, 64, 32, 0, 128, 1, 2, 3, 0, 255, 255, 255, 255,
        ];
        assert_eq!(
            straight_top_down(2, &bottom_up),
            [
                0, 0, 0, 0, 255, 255, 255, 255, 10, 20, 30, 255, 128, 64, 0, 128
            ]
        );
    }
}
