# windowing

The platform layer of CrownUI: a platform-neutral contract at the crate root and backends as
feature-gated modules.

| Item | Purpose |
|---|---|
| `Platform` | create, look up and close windows; report `Capabilities`; hand out render surfaces; clipboard text (`set_clipboard_text`, `request_clipboard_text` → `WindowEvent::ClipboardText`) and rich contents (`set_clipboard(items)` in several MIME types, `read_clipboard(ClipboardReadId, mime_types)` → `WindowEvent::ClipboardRead` with the first type held); drag and drop (`start_drag(window, DragSource)` with the `DndActions` it allows → `WindowEvent::DragEnded(DragOutcome::Dropped(DndAction))`, `accept_drag(Option<&DropAcceptance>)` answering `DragEnter`/`DragOver` with a MIME type and preferred `DndAction`, then `Drop` carrying the settled action); app icons for launchers (`set_app_icon(window, app_id, AppIconPlacement)`, `remove_app_icon`, `activate_app_icon` → `WindowEvent::LaunchApp` when the app has no windows; `WindowEvent::AppWindows(AppWindows)` reports each app's window count and focus, `WindowEvent::AppIconLifted` while the platform draws an icon itself) |
| `WindowControl` | per-window operations (title, cursor, redraw, material, interactive move/resize, `set_text_input`, `set_idle_inhibited`, `set_layer_height`) and `is_configured`, false until the first configure is acknowledged |
| `LayerOptions` (`Layer`, `Anchor`, `LayerExtent`, `LayerMargin`, `KeyboardInteractivity`) | `WindowAttributes::with_layer` makes a desktop-shell surface (panel, dock, overlay) instead of a toplevel; `LayerExtent::Output` sizes an axis to the output without anchoring both edges, so a top panel can keep room for popups below while reserving only its own height |
| `TextInputState`, `TextInputPurpose` | what the input method sees of the focused field: caret area, surrounding text, content purpose |
| `PlatformSurface` | `Send` handle for the render thread: frame pacing and `SurfaceState` commits; `FrameTiming` carries the presentation time and refresh interval when known, and whether the wait blocked for the callback |
| `ApplicationHandler<P>` | callbacks a backend's event loop drives |
| `WindowEvent` | neutral events built on the `input` crate (DOM-style names) |

Backends:
- `wayland` (default feature): CrownOS-first; `wayland::EventLoop` runs a `wayland::WaylandPlatform`.
  CrownOS-only protocols (screencast, input capture, virtual outputs) live on `WaylandPlatform`
  and arrive as `wayland::WaylandEvent`. `crownos_app_icon_manager_v1` backs the `Platform` app icon
  methods (`app_icons`): one `crownos_app_icon_v1` per window and app id, destroyed when removed or
  when the window closes; its `launch` arrives as `CrownEvent::AppIconLaunch` and
  `WindowEvent::LaunchApp`, its `lifted`/`landed` as `WindowEvent::AppIconLifted` at the icon's
  window. From v2 the manager's `app_windows` is kept per app (dropped at count 0) and delivered as
  `WindowEvent::AppWindows` to every window, replayed to each window created later.
  `crownos_background_effects` (v1–2) renders `Material`s (`materials`); a v2 manager's
  `composited_blur` capability is `composited_materials` (the compositor lays a layer surface on its
  glass and cuts both with one edge), and `CrownEvent::MaterialCapabilitiesChanged` reports changes.
  Standard protocols, each bound only when the compositor offers it (the matching `Capabilities`
  flag says which were found):

  | Protocol | Surfaces as |
  |---|---|
  | `zwp_pointer_gestures_v1` (swipe, pinch, hold from v3) | `WindowEvent::TouchpadGesture` at the window under the pointer when the gesture began; `touchpad_gestures` |
  | `zwp_text_input_v3` | `WindowControl::set_text_input` enables it for the keyboard-focused window; preedit, commit and delete apply together on `done` as `WindowEvent::TextInput`; `text_input` |
  | `wl_data_device` | clipboard text and drag and drop; pasting and dropping read the pipe through a calloop fd source, copying and dragging write from a short-lived thread. A drag's icon is a `wl_shm` surface the compositor moves (sized by `wp_viewporter` when bound); a drop is accepted as copy or move (plus ask when preferred), preferring the target's action; the compositor's `wl_data_offer.action`/`wl_data_source.action` choice is reported on the drop and the drag's end (an asked drop finishes as copy). A selection offers every MIME type it holds, a text item under every text type |
  | `wp_presentation` | `RenderSurface` asks for feedback on every present; `FrameTiming::{presented_at, refresh}`; `presentation_time` |
  | `zwlr_layer_shell_v1` (v1–4) | layer windows; configure → `Resized`, `closed` → `CloseRequested`; output-sized layers follow output changes; `layer_shell` |
  | `zwp_idle_inhibit_manager_v1` | `WindowControl::set_idle_inhibited` holds an inhibitor on the window's surface; `idle_inhibit` |
  | `wp_color_management_v1` | `WaylandPlatform::tag_window_srgb` tags a `wl_shm` window sRGB (gamma 2.2). Opt-in: Vulkan/EGL WSI tag their surface themselves and a second tag is a protocol error |
- `headless`: in-memory windows, a `ManualClock` and injected events for tests and previews.

Adding a backend: create a feature-gated module that implements `Platform`, `WindowControl` and
`PlatformSurface`, plus a runner that drives `ApplicationHandler`. Nothing above this crate changes.

Invariants: no `Mutex`, `Arc` or atomics in this crate's code; window state is owned by the
platform value and reached through `&mut`. Nothing presents to a window before `is_configured`:
the runtime holds the render thread's viewport until then, so no buffer is attached early.
