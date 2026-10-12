# impeller

CrownUI's GPU backend on [Impeller](https://docs.flutter.dev/perf/impeller) (Flutter's renderer,
via `impellers =0.4.3`). It implements the `graphics` contract for Wayland surfaces.

## Public API
- `ImpellerBackend`: `GraphicsBackend` + `CreateRenderer<S>` for any
  `S: HasWindowHandle + HasDisplayHandle` with Wayland handles.
- `ImpellerTextSystem`: one `TypographyContext` (≈160 ms to create, it scans the system fonts).
- `ImpellerTextLayout`: a shaped `Paragraph` plus metrics read once; `Send`.
- `ImpellerRenderer`: caches paragraphs, textures and one `DisplayList` per boundary; `render`
  composes them into a frame. Extras: `api()`, `frames_presented()`, `presented_size()`.
- `RenderApi`: `Vulkan` (default) or `Gles` (`CROWNUI_RENDERER=gles`, or when Vulkan fails);
  offscreen renderers report `GlesSurfaceless` or `GlesPbuffer`.
- `Offscreen { width, height, scale }`: `CreateRenderer<Offscreen>` renders headless into an
  4x MSAA RGBA8 + stencil8 FBO on the EGL surfaceless platform (pbuffer fallback), no window or
  display. `ImpellerRenderer::read_pixels()` resolves it and returns `Pixels` (straight alpha, top row first);
  `encode_png(&Pixels)` encodes them.
- `unsafe select_vulkan_driver(main_device)`: sets `VK_LOADER_DRIVERS_SELECT` to the Vulkan
  driver of the compositor's GPU (a `dev_t`), so other GPUs' drivers are never loaded
  (≈50 MiB less RSS on hybrid laptops) and those GPUs stay asleep. User choices
  (`VK_*`, `DRI_PRIME`, `__NV_PRIME_RENDER_OFFLOAD`) win. Call it before any thread starts.
- Feature `static` links `libimpeller.a` instead of the shared library.

## Invariants
- Every target antialiases with 4x MSAA: Vulkan through Impeller's swapchain, the GLES window
  through its EGL config, offscreen through multisampled renderbuffers. GLES falls back to one
  sample when the driver offers no multisampled config.
- Blur radii follow CSS: `Paint::blur` and backdrop blur use σ = radius; `Shadow` blur uses
  σ = radius / 2 (`box-shadow`).
- Rounded rects, clips and shadows follow the renderer's `CornerShape`: `Round` uses Impeller's
  rounded rects; `Squircle` draws paths matching crownpositor's `rounded_box` (a quarter of
  `x⁴ + y⁴ = 1` over 1.5× the radius, capped at half the shorter side), one cubic per corner.
- Uploaded images are straight alpha and get premultiplied; colors are passed straight.
- Text width is the widest line, or the paragraph width for centered/trailing/justified/RTL text
  (unconstrained text of that kind is relaid out at its intrinsic width first).
- The renderer is `!Send`: create, use and drop it on one thread (GLES contexts are
  thread-bound). The caller keeps the window surface alive longer than the renderer.
- Teardown: swapchain (it owns the `VkSurfaceKHR`) → lists, paragraphs, textures → context.
- Present blocks for vsync (Vulkan FIFO, EGL swap interval 1). A frame without a surface
  (resize race) is skipped, not an error.

- Offscreen output is deterministic per GPU driver, not across drivers (NVIDIA and Mesa differ
  by antialiasing). Pin the EGL vendor for golden tests, e.g.
  `__EGL_VENDOR_LIBRARY_FILENAMES=/usr/share/glvnd/egl_vendor.d/50_mesa.json`.

## Why there is process-global state
Impeller creates Wayland swapchains 1x1 because Wayland reports no surface extent. The Vulkan
proc-address callback interposes `vkGetPhysicalDeviceSurfaceCapabilitiesKHR`,
`vkCreateSwapchainKHR` and `vkAcquireNextImageKHR`; those are plain `extern "system"` functions
with no user data, so the wanted extent per `VkSurfaceKHR` lives in a fixed table of atomics
(32 surfaces) and the real function pointers in `OnceLock`s. No locks are taken.

## Non-goals
X11, Windows and macOS surfaces; letter spacing (Impeller's C API has no setter, the field is
ignored); sharing one GPU context between windows; Vulkan offscreen rendering.
