/// How a camera encodes the frames it sends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Encoding {
    /// Motion JPEG: what UVC cameras send at HD sizes and full frame rate.
    Mjpeg,
    /// Packed 4:2:2 YUV.
    Yuyv,
    /// Semi-planar 4:2:0 YUV.
    Nv12,
}

/// The encoding and size a camera stream settled on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CaptureFormat {
    /// How frames arrive from the camera.
    pub encoding: Encoding,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}
