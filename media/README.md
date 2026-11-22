# media

The media contracts that CrownUI's `camera`, effect plugins and app recorders share. See
`docs/media.md` for the agreement with the audio side.

## Public API
- `VideoFrame { pts, width, height, rgba }`: one decoded frame as opaque, tightly packed RGBA8.
  `VideoFrame::new` fails with `MediaError::FrameSize` (`CUI0801`) unless the buffer holds exactly
  `width * height` pixels. `into_rgba()` hands the buffer to a consumer without a copy.
- `ClockNanos`, re-exported from `audio`: the PipeWire graph clock (`CLOCK_MONOTONIC` ns). Video
  and audio frames carry it, so recorders mux them without translating timestamps.

## Invariants
- Plain data only: no threads, no PipeWire, no GPU types.
- Every frame is opaque, so renderers upload it without premultiplying.

## Non-goals
Capture (`camera`), encoding (`crownos-media-encode`), and GPU-resident frames (a dma-buf variant
comes with the zero-copy preview).
