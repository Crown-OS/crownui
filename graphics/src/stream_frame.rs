use crate::GraphicsError;

const BYTES_PER_PIXEL: usize = 4;

/// One frame of a live image such as a camera preview: opaque RGBA8, tightly packed, top row
/// first. Opaque pixels need no premultiplication, so a renderer uploads the buffer as it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamFrame {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl StreamFrame {
    /// Wraps `rgba`, whose every alpha byte is 255; fails unless it holds exactly
    /// `width * height` pixels.
    pub fn new(width: u32, height: u32, rgba: Vec<u8>) -> Result<Self, GraphicsError> {
        let expected = width as usize * height as usize * BYTES_PER_PIXEL;
        if width == 0 || height == 0 || rgba.len() != expected {
            return Err(GraphicsError::Image(format!(
                "expected {expected} bytes for a {width}x{height} frame, got {}",
                rgba.len()
            )));
        }
        Ok(Self {
            width,
            height,
            rgba,
        })
    }

    /// Width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The pixels.
    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }

    /// The pixel buffer, e.g. to hand it to a GPU upload without a copy.
    pub fn into_rgba(self) -> Vec<u8> {
        self.rgba
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frame_holds_exactly_its_pixels() {
        assert!(StreamFrame::new(2, 1, vec![0; 8]).is_ok());
        assert!(StreamFrame::new(2, 1, vec![0; 7]).is_err());
        assert!(StreamFrame::new(0, 0, Vec::new()).is_err());
    }
}
