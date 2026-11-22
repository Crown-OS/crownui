use media::VideoFrame;
use turbojpeg::Decompressor;

use crate::{Encoding, capture::EncodedFrame};

use super::{DecodeError, jpeg, yuv};

/// Turns encoded camera frames into opaque RGBA [`VideoFrame`]s.
pub(crate) struct Decoder {
    jpeg: Decompressor,
}

impl Decoder {
    pub(crate) fn new() -> Result<Self, DecodeError> {
        Ok(Self {
            jpeg: Decompressor::new()?,
        })
    }

    pub(crate) fn decode(&mut self, frame: &EncodedFrame) -> Result<VideoFrame, DecodeError> {
        let (width, height, rgba) = match frame.format.encoding {
            Encoding::Mjpeg => jpeg::decode(&mut self.jpeg, &frame.bytes)?,
            Encoding::Yuyv => {
                let (width, height) = (frame.format.width, frame.format.height);
                (
                    width,
                    height,
                    yuv::yuyv_to_rgba(&frame.bytes, frame.stride, width, height)?,
                )
            }
            Encoding::Nv12 => {
                let (width, height) = (frame.format.width, frame.format.height);
                (
                    width,
                    height,
                    yuv::nv12_to_rgba(&frame.bytes, frame.stride, width, height)?,
                )
            }
        };
        Ok(VideoFrame::new(frame.pts, width, height, rgba)?)
    }
}
