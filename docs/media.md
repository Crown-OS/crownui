# Media: audio and video seams

This doc is where the voice team (`audio`, crown-voiced) and the camera team (`media`, `camera`)
meet. It records what both sides agreed on 2026-11-04. Change it only with both teams.

## Crates and layers
| Crate | Layer | Owner | Role |
|---|---|---|---|
| `audio` | foundations | voice | Microphone capture over PipeWire, optional crown-voiced enhancement, and `ClockNanos` |
| `media` | contracts | camera | Shared media contracts; depends on `audio` and re-exports `ClockNanos` |
| `camera` | implementations | camera | PipeWire camera capture |
| `camera_preview` | components | camera | The preview widget |

Each side adds its own `layers.toml` row, facade feature (`audio`, `video`), `BundledCrate` entry
and `xtask/bundle/manifest.toml` dependencies, in separate commits. On conflicts in
`bundled_crate.rs`, `manifest.toml` or `crownui/Cargo.toml`, rebase. Neither feature is on by
default, so apps without a microphone or camera link no PipeWire.

## Error codes
`CUI07xx` covers audio (`docs/errors.md`). `CUI08xx` is reserved for video.

## The media clock
- `audio::ClockNanos(pub u64)` is the PipeWire graph clock in nanoseconds (`CLOCK_MONOTONIC`, read
  from the stream's `pw_time.now` each cycle).
- Every `AudioFrame` carries the time of its first sample as `pts`. Video frames carry the same
  type, so a recorder muxes audio and video directly, with no clock translation.
- An audio frame is 480 mono `f32` samples at 48 kHz (10 ms). Its pts advances by
  `samples * 1e9 / 48000` within a PipeWire buffer.

## Microphone access
- Camera records audio through `crownui::audio::Microphone` and drops its own microphone code, so
  camera videos get the same enhancement for free.
- `MicrophoneOptions::default()` captures the default source with `Enhancement::NoiseSuppression`.
  If crown-voiced is absent, the stream captures the raw source and reports
  `StreamState::Unenhanced(reason)`.
- Device permission is deferred. The plan is one `crownos-device-access-v1` protocol with
  `DeviceKind { Microphone, Camera }`, modelled on `request_agent_access`.

## Shared pieces outside crownui
| Seam | Agreement | Owner |
|---|---|---|
| Hardware inventory and AI packages | `crownos-accel`, `/etc/crownos/ai.ron` (written by the installer; schema in `crownos-config/src/system/ai.rs`), `crownos-ai-<vendor>` metas, one installer toggle | camera owns the installer, metas and ONNX Runtime packages |
| ONNX Runtime | One CPU core `onnxruntime-crownos` plus additive providers `onnxruntime-crownos-{cuda,migraphx,openvino,vitisai}`; crown-voiced ships one `onnx` backend | camera |
| Encoding and muxing | Extract screencapture's `record/{audio/encoder,mux}.rs` into a shared `crownos-media-encode` (audio: Opus/FLAC; video added by camera) | whoever starts first |
| Lazy backend loading | C-ABI vtable, `libloading` and idle unload; consider a shared `crownos-accel::plugin` once both sides have an implementation | later |
