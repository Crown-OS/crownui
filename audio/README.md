# audio

Microphone access for CrownOS apps. It captures through PipeWire and, when asked, through the
AI enhancement that `crown-voiced` serves. The crate does not depend on `Cx`: each stream owns a
PipeWire thread, and the app reaches it only through lock-free rings and `tokio::sync::watch`
channels.

```rust,no_run
use audio::{Enhancement, Microphone, MicrophoneOptions};

let mut stream = Microphone::open(
    MicrophoneOptions::default().enhancement(Enhancement::NoiseSuppression),
)?;
let mut frames = stream.take_frames().expect("taken once");
std::thread::spawn(move || {
    while let Ok(frame) = frames.pop() {
        let _ = (frame.pts, frame.samples);
    }
});
println!("{:?}", *stream.state().borrow());
# Ok::<(), audio::AudioError>(())
```

## Public API
- `Microphone::open(MicrophoneOptions) -> Result<MicrophoneStream, AudioError>` returns once
  PipeWire accepts the stream. `MicrophoneOptions::default()` captures the default source with
  `Enhancement::NoiseSuppression`. Set `.source(SourceId)` and `.enhancement(Enhancement)` to
  change that.
- `MicrophoneStream` is `Send` and stops when dropped. It offers:
  - `take_frames() -> Option<FrameConsumer>`: the `rtrb` ring of `AudioFrame`s, handed over once.
  - `level()` and `state()`: `watch::Receiver<Level>` and `watch::Receiver<StreamState>`.
  - `set_enhancement(Enhancement)`: switches enhancement without stopping the stream.
- `AudioFrame { pts: ClockNanos, samples: [f32; FRAME_SAMPLES] }` holds 10 ms of mono audio at
  `SAMPLE_RATE` (48 kHz).
- `ClockNanos(pub u64)` is the PipeWire graph clock (`CLOCK_MONOTONIC` ns). Video frames carry the
  same clock (see `docs/media.md`).
- `Level { peak, rms }` holds linear amplitudes per frame. `Level::measure(&[f32])` computes it.
- `StreamState` is one of `Starting`, `Live { enhanced_by }`, `Unenhanced(UnenhancedReason)` or
  `Failed(String)`.
- `sources() -> watch::Receiver<Vec<SourceInfo>>` lists the Audio/Source nodes with hotplug.
  Each `SourceInfo` has `id: SourceId` (the `node.name`), `description` and `is_default`.
  crown-voiced's own `crownos-voice.*` nodes are left out.
- `Enhancement` (`Off`, `Light`, `NoiseSuppression`, `StudioVoice`) is re-exported from
  `crownos-voice-protocol`.
- `AudioError` carries the `CUI07xx` codes listed in `docs/errors.md`.

## Invariants
- The real-time callback does not allocate or lock. It packs the PipeWire buffers into
  fixed-size frames, pushes them into the ring and publishes the level. When the ring is full,
  it drops the frame.
- With enhancement on, the stream asks crown-voiced (`crownos-voice` over `crownos-ipc`) to
  `open_stream` and captures the returned node with `target.object`. If the daemon is absent,
  fails or does not answer within 3 s, the stream captures the raw source and reports
  `StreamState::Unenhanced`. It follows `StreamChanged` when the daemon switches backend.
  Dropping the stream closes the connection, and the daemon then tears the node down.
- There is no `Mutex`, `RefCell` or `Rc`. Each PipeWire thread owns its state and drains one
  event queue.

## Non-goals
Playback, encoding (see `crownos-media-encode`), multichannel capture, and device permissions
(the `crownos-device-access-v1` protocol, to come).
