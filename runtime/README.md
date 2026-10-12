# runtime

Runs CrownUI apps. Everything here is generic over the three contracts, so swapping a backend is a
type parameter change:

- `Runner<G: GraphicsBackend, L: LayoutEngine>` implements `windowing::ApplicationHandler<P>` for
  any `P: Platform` whose surface `G` can render to. The UI thread (events, signals, layout, text)
  runs in the platform's event loop; a dedicated **render thread** owns the GPU renderer and the
  `RenderTree`, applies `RenderOp`s, runs every animation on its own clock, re-records only changed
  repaint boundaries, composes, commits `SurfaceState` together with the frame and presents.
  A stalled UI thread never stops an animation.
- `InlineApp<G, L>` drives the same pipeline synchronously with a manual clock (`advance`,
  `settle`); `crowntest` and previews use it with `graphics::RecordingBackend`.
- `AppConfig::min_size` keeps a toplevel from shrinking below what its layout needs; `cx.release_image`
  frees an uploaded image once no node, exiting ones included, still draws it.
- `AppConfig::layer(LayerOptions)` runs the app as a layer surface; `cx.request_animation_frame` callbacks
  run once per display refresh (paced by the refresh interval the render thread reports); the root's
  `pointer_events_none` region and `cx.set_idle_inhibited` reach the window with each frame.
- `AppConfig::corner_shape(CornerShape::Squircle)` draws every rounded corner in the window, kit
  widgets' included, as the compositor's squircle; `InlineApp::set_corner_shape` does the same and
  re-records every boundary.
- Bridges (features): `tokio` runs `cx.spawn` futures, `mcp` serves agents through the in-app MCP
  server, `accessibility` keeps AT-SPI screen readers in sync.

Threads talk only through channels; nothing is shared or locked.

Diagnostics: CUI0401 image rejected · CUI0402 renderer creation failed · CUI0403 surface lost ·
CUI0404 frame failed · CUI0405 render thread stopped · CUI0406 no surface · CUI0407 no executor ·
CUI0408 MCP unavailable.
