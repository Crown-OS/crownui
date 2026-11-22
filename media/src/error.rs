use thiserror::Error;

/// Why a media value could not be built.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MediaError {
    /// The pixel buffer does not hold `width * height` RGBA pixels.
    #[error("CUI0801: a {width}x{height} frame needs {expected} bytes, got {actual}")]
    FrameSize {
        /// Width in pixels.
        width: u32,
        /// Height in pixels.
        height: u32,
        /// Bytes the size needs.
        expected: usize,
        /// Bytes given.
        actual: usize,
    },
}
