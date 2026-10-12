/// Copies straight-alpha RGBA8 pixels into the premultiplied form Impeller samples.
pub(crate) fn premultiplied_rgba(straight: &[u8]) -> Vec<u8> {
    let multiply =
        |channel: u8, alpha: u8| ((u16::from(channel) * u16::from(alpha) + 127) / 255) as u8;
    straight
        .chunks_exact(4)
        .flat_map(|pixel| {
            let alpha = pixel[3];
            [
                multiply(pixel[0], alpha),
                multiply(pixel[1], alpha),
                multiply(pixel[2], alpha),
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
        .flat_map(|row| row.chunks_exact(4))
        .flat_map(|pixel| match pixel[3] {
            0 => [0; 4],
            alpha => [
                divide(pixel[0], alpha),
                divide(pixel[1], alpha),
                divide(pixel[2], alpha),
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
