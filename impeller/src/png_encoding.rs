use graphics::GraphicsError;

use crate::Pixels;

/// Encodes `pixels` as an 8-bit RGBA PNG.
pub fn encode_png(pixels: &Pixels) -> Result<Vec<u8>, GraphicsError> {
    let encoding_error =
        |error: png::EncodingError| GraphicsError::Image(format!("PNG encoding failed: {error}"));
    let mut encoded = Vec::new();
    let mut encoder = png::Encoder::new(&mut encoded, pixels.width, pixels.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(encoding_error)?;
    writer
        .write_image_data(&pixels.rgba)
        .map_err(encoding_error)?;
    writer.finish().map_err(encoding_error)?;
    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pngs_start_with_the_signature_and_reject_wrong_sizes() {
        let pixels = Pixels {
            width: 2,
            height: 1,
            rgba: vec![255, 0, 0, 255, 0, 0, 255, 128],
        };
        let Ok(encoded) = encode_png(&pixels) else {
            panic!("a 2x1 image encodes");
        };
        assert!(encoded.starts_with(b"\x89PNG\r\n\x1a\n"));
        let truncated = Pixels {
            rgba: vec![0; 4],
            ..pixels
        };
        assert!(encode_png(&truncated).is_err());
    }
}
