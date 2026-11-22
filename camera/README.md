# camera

Camera capture for CrownOS apps. It captures through PipeWire, which fronts libcamera and V4L2
alike, and decodes every frame into an opaque RGBA `media::VideoFrame` stamped on the audio
clock. The crate does not depend on `Cx`: each stream owns a PipeWire thread and a decoding
thread, and the app reaches them only through channels and `tokio::sync::watch`.

```rust,no_run
use camera::{Camera, CameraOptions};

let stream = Camera::open(CameraOptions::default().size(1280, 720), |frame| {
    let _ = (frame.pts(), frame.width(), frame.rgba());
})?;
let tap = stream.tap(4);
std::thread::spawn(move || {
    while let Ok(frame) = tap.recv() {
        let _ = frame.into_rgba();
    }
});
println!("{:?}", *stream.state().borrow());
# Ok::<(), camera::CameraError>(())
```

## Public API
- `Camera::open(CameraOptions, preview)` returns a `CameraStream` once PipeWire accepts the
  stream. `preview: FnMut(VideoFrame)` runs on the decoding thread for every frame, which is where
  the kit's preview pushes frames to an `ImageStream`. `CameraOptions::default()` captures the
  default camera at 1920×1080, 30 fps. Set `.camera(CameraId)`, `.size(w, h)` and
  `.frame_rate(fps)` to change that; the camera answers with its nearest mode.
- `CameraStream` is `Send` and stops when dropped. It offers:
  - `state() -> watch::Receiver<CameraState>`: `Starting`, `Live(CaptureFormat)` or `Failed(String)`.
  - `tap(capacity) -> FrameTap`: a bounded receiver of frame copies for recorders, photos and
    effects. Frames are copied only while a tap is alive, and dropped while it is full.
- `cameras() -> watch::Receiver<Vec<CameraInfo>>` lists the `Video/Source` nodes with the
  `Camera` role, with hotplug. Each `CameraInfo` has `id: CameraId` (the `node.name`) and
  `description`.
- `CaptureFormat { encoding: Encoding, width, height }` is what the camera sends; `Encoding` is
  `Mjpeg`, `Yuyv` or `Nv12`.
- `CameraError` carries the `CUI08xx` codes listed in `docs/errors.md`.

## Invariants
- The real-time callback only copies the buffer into recycled memory and hands it off. When the
  decoder falls behind, it drops the frame.
- MJPEG is decoded by libjpeg-turbo. YUYV and NV12 are converted as BT.601 limited range, which is
  what UVC cameras send.
- The stream asks for buffers in CPU memory (`MemPtr`/`MemFd`). V4L2 nodes otherwise offer
  dma-bufs, which fail the link.
- There is no `Mutex`, `RefCell` or `Rc`. Each thread owns its state and talks through channels.

## Non-goals
GPU decoding and dma-buf frames (to come with the zero-copy preview), effects (the effect host
comes next), encoding (`crownos-media-encode`), and device permissions (`crownos-device-access-v1`).
