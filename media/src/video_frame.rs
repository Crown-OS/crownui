use crate::{ClockNanos, MediaError};

const BYTES_PER_PIXEL: usize = 4;

/// One decoded video frame: opaque RGBA8 (every alpha byte is 255), tightly packed, top row
/// first, stamped on the same clock as [`audio::AudioFrame`]s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoFrame {
    pts: ClockNanos,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl VideoFrame {
    /// Wraps `rgba`, which must hold exactly `width * height` pixels.
    pub fn new(
        pts: ClockNanos,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    ) -> Result<Self, MediaError> {
        let expected = width as usize * height as usize * BYTES_PER_PIXEL;
        if width == 0 || height == 0 || rgba.len() != expected {
            return Err(MediaError::FrameSize {
                width,
                height,
                expected,
                actual: rgba.len(),
            });
        }
        Ok(Self {
            pts,
            width,
            height,
            rgba,
        })
    }

    /// When the frame was captured.
    pub fn pts(&self) -> ClockNanos {
        self.pts
    }

    /// Width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Bytes per row.
    pub fn stride(&self) -> u32 {
        self.width * BYTES_PER_PIXEL as u32
    }

    /// The pixels.
    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }

    /// The pixel buffer, for a consumer that takes ownership (a GPU upload).
    pub fn into_rgba(self) -> Vec<u8> {
        self.rgba
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frame_holds_exactly_its_pixels() {
        let frame = VideoFrame::new(ClockNanos(7), 2, 1, vec![0; 8]).expect("2x1 frame");
        assert_eq!((frame.width(), frame.height(), frame.stride()), (2, 1, 8));
        assert_eq!(frame.pts(), ClockNanos(7));
        assert!(matches!(
            VideoFrame::new(ClockNanos(0), 2, 1, vec![0; 6]),
            Err(MediaError::FrameSize {
                expected: 8,
                actual: 6,
                ..
            })
        ));
    }
}
