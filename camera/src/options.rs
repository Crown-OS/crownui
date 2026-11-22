use crate::CameraId;

const DEFAULT_WIDTH: u32 = 1920;
const DEFAULT_HEIGHT: u32 = 1080;
const DEFAULT_FRAME_RATE: u32 = 30;

/// What [`Camera::open`](crate::Camera::open) asks for: the default camera at 1080p30 unless told
/// otherwise. The camera answers with the nearest mode it has.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraOptions {
    pub(crate) camera: Option<CameraId>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) frame_rate: u32,
}

impl Default for CameraOptions {
    fn default() -> Self {
        Self {
            camera: None,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            frame_rate: DEFAULT_FRAME_RATE,
        }
    }
}

impl CameraOptions {
    /// Captures `camera` instead of the system default.
    pub fn camera(self, camera: CameraId) -> Self {
        Self {
            camera: Some(camera),
            ..self
        }
    }

    /// Asks for frames of `width` by `height` pixels.
    pub fn size(self, width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ..self
        }
    }

    /// Asks for `frames_per_second`.
    pub fn frame_rate(self, frames_per_second: u32) -> Self {
        Self {
            frame_rate: frames_per_second,
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_the_default_camera_at_1080p30() {
        let options = CameraOptions::default();
        assert_eq!(options.camera, None);
        assert_eq!(
            (options.width, options.height, options.frame_rate),
            (1920, 1080, 30)
        );
    }

    #[test]
    fn builders_set_camera_size_and_rate() {
        let options = CameraOptions::default()
            .camera(CameraId::new("v4l2_input.usb"))
            .size(1280, 720)
            .frame_rate(60);
        assert_eq!(options.camera, Some(CameraId::new("v4l2_input.usb")));
        assert_eq!(
            (options.width, options.height, options.frame_rate),
            (1280, 720, 60)
        );
    }
}
