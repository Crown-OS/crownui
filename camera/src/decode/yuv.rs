//! BT.601 limited-range YUV to RGBA, the colorimetry UVC cameras send.

use super::DecodeError;

const BYTES_PER_PIXEL: usize = 4;
const OPAQUE: u8 = 255;

fn clamp(value: i32) -> u8 {
    value.clamp(0, 255) as u8
}

/// One pixel in fixed point (8 fractional bits).
fn rgba(y: u8, u: u8, v: u8) -> [u8; 4] {
    let luma = 298 * (i32::from(y) - 16) + 128;
    let (u, v) = (i32::from(u) - 128, i32::from(v) - 128);
    [
        clamp((luma + 409 * v) >> 8),
        clamp((luma - 100 * u - 208 * v) >> 8),
        clamp((luma + 516 * u) >> 8),
        OPAQUE,
    ]
}

fn require(bytes: &[u8], expected: usize) -> Result<(), DecodeError> {
    if bytes.len() < expected {
        return Err(DecodeError::ShortFrame {
            expected,
            actual: bytes.len(),
        });
    }
    Ok(())
}

/// Packed `Y0 U Y1 V` rows of `stride` bytes (0 for tightly packed).
pub(super) fn yuyv_to_rgba(
    bytes: &[u8],
    stride: u32,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, DecodeError> {
    let (width, height) = (width as usize, height as usize);
    let stride = (stride as usize).max(width * 2);
    require(bytes, stride * (height - 1) + width * 2)?;
    let mut out = vec![0; width * height * BYTES_PER_PIXEL];
    for (source, target) in bytes
        .chunks(stride)
        .zip(out.chunks_exact_mut(width * BYTES_PER_PIXEL))
    {
        for (quad, pair) in source[..width * 2]
            .chunks_exact(4)
            .zip(target.chunks_exact_mut(2 * BYTES_PER_PIXEL))
        {
            let (y0, u, y1, v) = (quad[0], quad[1], quad[2], quad[3]);
            pair[..4].copy_from_slice(&rgba(y0, u, v));
            pair[4..].copy_from_slice(&rgba(y1, u, v));
        }
    }
    Ok(out)
}

/// A luma plane of `stride`-byte rows followed by interleaved `U V` rows at half height.
pub(super) fn nv12_to_rgba(
    bytes: &[u8],
    stride: u32,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, DecodeError> {
    let (width, height) = (width as usize, height as usize);
    let stride = (stride as usize).max(width);
    let chroma_offset = stride * height;
    require(
        bytes,
        chroma_offset + stride * (height.div_ceil(2) - 1) + width,
    )?;
    let (luma, chroma) = bytes.split_at(chroma_offset);
    let mut out = vec![0; width * height * BYTES_PER_PIXEL];
    for (row, target) in out.chunks_exact_mut(width * BYTES_PER_PIXEL).enumerate() {
        let luma_row = &luma[row * stride..row * stride + width];
        let chroma_row = &chroma[(row / 2) * stride..];
        for (column, pixel) in target.chunks_exact_mut(BYTES_PER_PIXEL).enumerate() {
            let pair = column & !1;
            pixel.copy_from_slice(&rgba(
                luma_row[column],
                chroma_row[pair],
                chroma_row[pair + 1],
            ));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limited_range_black_and_white_map_to_full_range() {
        assert_eq!(rgba(16, 128, 128), [0, 0, 0, 255]);
        assert_eq!(rgba(235, 128, 128), [255, 255, 255, 255]);
    }

    #[test]
    fn saturated_red_stays_red() {
        let [red, green, blue, _] = rgba(81, 90, 240);
        assert!(red > 250 && green < 5 && blue < 5);
    }

    #[test]
    fn yuyv_rows_respect_the_stride() {
        let padded_row = [235, 128, 16, 128, 0, 0];
        let bytes = [padded_row, padded_row].concat();
        let rgba = yuyv_to_rgba(&bytes, 6, 2, 2).expect("2x2 frame");
        assert_eq!(&rgba[..8], &[255, 255, 255, 255, 0, 0, 0, 255]);
        assert_eq!(rgba.len(), 16);
    }

    #[test]
    fn nv12_shares_one_chroma_pair_between_four_pixels() {
        let bytes = [235, 235, 16, 16, 128, 128];
        let rgba = nv12_to_rgba(&bytes, 2, 2, 2).expect("2x2 frame");
        assert_eq!(&rgba[..4], &[255, 255, 255, 255]);
        assert_eq!(&rgba[12..], &[0, 0, 0, 255]);
    }

    #[test]
    fn a_short_buffer_is_rejected() {
        assert!(matches!(
            yuyv_to_rgba(&[0; 3], 0, 2, 1),
            Err(DecodeError::ShortFrame {
                expected: 4,
                actual: 3
            })
        ));
    }
}
