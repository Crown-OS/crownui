# Impeller on CrownOS (spike results, 2026-10-02)

Measured on AMD Radeon 780M (RADV, Mesa 26.2.3) under crownpositor at 144 Hz with `impellers =0.4.3`
(Flutter engine commit `5d33e22e192980309c55b1b25bde7f984284dca9`, API 1.4.0).

| Backend | Verdict | Notes |
|---|---|---|
| Vulkan | **go, with the extent shim** | swapchain is created 1x1 on Wayland; the loader shim fixes it |
| GLES (EGL + wayland-egl) | go | `wrap_fbo` takes a size; context is thread-bound; EGL config needs stencil 8 |
| Typography | go | first `TypographyContext` costs ~160 ms (scans /usr/share/fonts); keep one |

Frame cost (acquire + draw + present, gradient/shadow/blur/paragraph/backdrop-blur scene):
Vulkan FIFO 6.85 ms avg (vsync wait in `acquire_next_surface_new`, draw 0.19 ms, present 0.05 ms,
non-blocking); Vulkan MAILBOX 1.25 ms; GLES 0.54 ms CPU unthrottled. A second swapchain on a
desync subsurface draws in 0.23 ms without blocking.

## Gotchas
1. **1x1 swapchain**: `ImpellerVulkanSwapchainCreateNew` hardcodes 1x1 and Wayland reports
   `currentExtent = 0xFFFFFFFF`. The `impeller` crate intercepts
   `vkGetPhysicalDeviceSurfaceCapabilitiesKHR`, `vkCreateSwapchainKHR` and `vkAcquireNextImageKHR`
   through the proc-address callback passed to `Context::new_vulkan`, reports the wanted extent and
   returns `VK_ERROR_OUT_OF_DATE_KHR` when it changes so Impeller recreates the swapchain.
2. The proc-address callback is only used during `Context::new_vulkan`; one Vulkan context per process.
3. Impeller hardcodes FIFO; `present()` returns immediately and the next acquire blocks.
4. Swapchain format is `R8G8B8A8_UNORM` (gamma-space blending like Flutter), premultiplied alpha.
5. Glyph cluster bounds come back with x/y swapped (upstream bug); text indices are UTF-16.
6. `create_new_vulkan_swapchain` takes ownership of the `VkSurfaceKHR`. Teardown: surface →
   swapchain → textures → context → `wl_surface`.
7. `GL Surface::present()` is a no-op; call `eglSwapBuffers`.
8. Thread safety: `DisplayList`, `Paint`, filters, `Path`, `Paragraph`, `Texture` are Send + Sync;
   `Context`, `VkSwapChain`, `Surface`, `TypographyContext`, `ParagraphBuilder` are neither.

## Linking
`features = ["prebuilt_libs", "cache_libs"]` downloads a 22.7 MB SDK into `.impeller_cache/`.
Set `IMPELLER_CACHE_DIR` to a vendored copy (`$DIR/linux_x64/lib/libimpeller.{so,a}`) for offline
builds. Enable `impellers/static_link` for shipping binaries (no rpath needed, +9 MiB).

The spike sources live outside the repo; the shim was ported into `impeller/src/vulkan/loader.rs`.
