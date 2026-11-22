use camera::{Camera, CameraOptions, CameraState, CameraStream, FrameTap, VideoFrame};
use geometry::Size;
use graphics::{ImageId, StreamFrame};
use reactive::{Runtime, Signal, Store};
use view::{Cx, ImageStream};

use crate::util::follow_watch::follow_watch;

/// A camera owned by the component that called [`use_camera`]: its state as a signal and its
/// frames as a live image, stopped when that component unmounts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CameraSignals {
    /// Starting, live in a format, or failed.
    pub state: Signal<CameraState>,
    /// The live image every decoded frame replaces; draw it with [`camera_preview`](super::camera_preview).
    pub image: ImageId,
    stream: Store<Option<CameraStream>>,
}

impl CameraSignals {
    /// A copy of every frame from now on, for a recorder, a photo or an effect; `None` when the
    /// camera failed to open.
    pub fn tap(self, cx: &mut Cx, capacity: usize) -> Option<FrameTap> {
        self.stream
            .try_update(cx, |stream| {
                stream.as_ref().map(|stream| stream.tap(capacity))
            })
            .flatten()
    }

    /// The size of the frames, once the camera is live.
    pub fn frame_size(self, runtime: &mut Runtime) -> Option<Size> {
        self.state.with(runtime, |state| match state {
            CameraState::Live(format) => Some(Size::new(format.width as f32, format.height as f32)),
            CameraState::Starting | CameraState::Failed(_) => None,
        })
    }
}

/// Opens a camera for the current component, streams its frames into a live image and mirrors
/// its state into a signal. A camera that cannot open shows as [`CameraState::Failed`].
pub fn use_camera(cx: &mut Cx, options: CameraOptions) -> CameraSignals {
    let live_image = cx.image_stream();
    let image = live_image.image();
    let (state, stream) = match Camera::open(options, move |frame| show(&live_image, frame)) {
        Ok(stream) => {
            let state = cx.signal(stream.state().borrow().clone());
            follow_watch(cx, state, stream.state());
            (state, Some(stream))
        }
        Err(error) => (cx.signal(CameraState::Failed(error.to_string())), None),
    };
    CameraSignals {
        state,
        image,
        stream: cx.store(stream),
    }
}

fn show(live_image: &ImageStream, frame: VideoFrame) {
    let (width, height) = (frame.width(), frame.height());
    if let Ok(frame) = StreamFrame::new(width, height, frame.into_rgba()) {
        let _ = live_image.push(frame);
    }
}
