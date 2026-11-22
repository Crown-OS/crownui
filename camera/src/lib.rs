#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod camera;
mod camera_id;
mod cameras;
mod capture;
mod decode;
mod error;
mod format;
mod options;
mod processing;
mod state;
mod stream;
mod util;

pub use camera::Camera;
pub use camera_id::{CameraId, CameraInfo};
pub use cameras::cameras;
pub use error::CameraError;
pub use format::{CaptureFormat, Encoding};
pub use media::{ClockNanos, VideoFrame};
pub use options::CameraOptions;
pub use state::CameraState;
pub use stream::{CameraStream, FrameTap};
