use thiserror::Error;

/// Why one captured frame could not be decoded; the stream skips it and goes on.
#[derive(Debug, Error)]
pub(crate) enum DecodeError {
    #[error("CUI0806: the JPEG decoder failed: {0}")]
    Jpeg(#[from] turbojpeg::Error),
    #[error("CUI0807: a frame needed {expected} bytes but arrived with {actual}")]
    ShortFrame { expected: usize, actual: usize },
    #[error(transparent)]
    Frame(#[from] media::MediaError),
}
