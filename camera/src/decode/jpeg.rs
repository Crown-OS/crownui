use turbojpeg::{Decompressor, Image, PixelFormat};

use super::DecodeError;

const BYTES_PER_PIXEL: usize = 4;

/// libjpeg-turbo's SIMD decoder, straight to RGBA (alpha 255); returns `(width, height, rgba)`.
pub(super) fn decode(
    decompressor: &mut Decompressor,
    jpeg: &[u8],
) -> Result<(u32, u32, Vec<u8>), DecodeError> {
    let header = decompressor.read_header(jpeg)?;
    let pitch = header.width * BYTES_PER_PIXEL;
    let mut rgba = vec![0; pitch * header.height];
    decompressor.decompress(
        jpeg,
        Image {
            pixels: rgba.as_mut_slice(),
            width: header.width,
            pitch,
            height: header.height,
            format: PixelFormat::RGBA,
        },
    )?;
    Ok((header.width as u32, header.height as u32, rgba))
}
