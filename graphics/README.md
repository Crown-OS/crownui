# graphics

The drawing contract between CrownUI and a GPU backend.

- `DrawList`: a flat list of `DrawCommand`s (rects, rounded rects, paths, shadows, text, images,
  clips, transforms, layers with backdrop blur) recorded by widgets and `Kind::paint`. It prints
  as stable text (`Display`), which is what snapshots, devtools and AI agents read and diff.
- `Paint` / `Brush`: solid colors, linear gradients, and `Brush::Image` — an uploaded image scaled
  into a rect and cropped by the shape it fills, so a rounded image needs no clip.
- `Composition`: per-frame list of `CompositeOp`s that places cached repaint boundaries with a
  transform, opacity, clip and backdrop blur. Composite-only changes never re-record a list.
- `TextStyle` / `TextSystem` / `TextLayout`: shaping and measuring on the UI thread; layouts are
  handed to the renderer by `TextLayoutId`.
- `GraphicsBackend`: bundles a text system and a renderer that agree on the layout type;
  `CreateRenderer<S>` builds a renderer for a surface type `S` (raw-window-handle for GPUs, nothing
  for the recording backend). `Renderer::set_corner_shape` picks the curve rounded rects, clips and
  shadows are drawn with; a backend that draws one shape keeps the default no-op.
- `RecordingBackend`: a dependency-free backend that keeps the last frame as text, used by
  `crowntest` and previews; its text system is a deterministic monospace approximation.

Invariants: no backend types leak through this crate; every command is plain data.
