# CrownUI vs other Linux toolkits

The same four scenarios are built in CrownUI, GPUI, Qt Quick, Electron and Tauri. One harness measures every app from the outside, the same way for each, so no toolkit reports its own numbers.

```sh
cargo xtask compare                                   # 5 runs × 10 s, every framework and scenario
cargo xtask compare --frameworks crownui,qt --scenarios list --runs 3 --duration 5
cargo xtask compare --placement desktop               # on the real display instead of the hidden output
cargo xtask compare --uncapped                        # frame pacing off: how many frames each toolkit can draw
cargo xtask compare --no-build                        # skip the build step
```

Results go to `results/<date>.md`, which is committed, and `results/<date>.json`, which is ignored and holds every run.

**Placement.** By default, every window opens on crownpositor's hidden `HEADLESS-1` output (`WORKSPACE=headless`). That output runs at 60 Hz and scale 1, is the same on every run, and keeps the windows out of your way. `--placement desktop` opens them on the focused workspace at the panel's refresh rate and scale instead, which is closer to what a user sees.

**Uncapped.** `--uncapped` turns frame pacing off where a toolkit allows it, so FPS shows how many frames it can draw rather than how many the display takes. CrownUI runs with `AppConfig::frame_pacing(FramePacing::Unpaced)`. GPUI has no switch, so the harness rebuilds it against a copy of the locked gpui crate with `apps/gpui/uncapped.patch`, which draws the next frame as soon as the last one is submitted instead of on the compositor's frame callback (in `target/gpui-uncapped`, apart from the unpatched build). Qt Quick, Chromium and WebKitGTK stayed at the compositor's pace with every switch tried (a swap interval of 0, Qt's basic render loop, `--disable-frame-rate-limit --disable-gpu-vsync`), so they run as usual. CPU ms per frame stays comparable across both modes.

**Before a run:** unlock the screen and stop other builds. A locked screen hides every window, including those on the headless output. crownpositor locks after its idle timeout and ignores logind inhibitors, so on the headless placement the harness keeps a small CrownUI window (a copy of the app, holding an idle inhibitor) on your workspace until the run ends. The compositor then throttles hidden windows to 1 Hz, and CrownUI stops drawing them entirely, so the harness rejects those runs. Wall time and CPU % move by 20–30% when another build is running. Instructions per frame are the stable figure.

## Scenarios
`apps/spec.toml` is the only source of sizes, counts and speeds. The harness passes them to every app as `BENCH_*` environment variables. The web apps receive them as a query string.

| Scenario | What the app does | What it shows |
|---|---|---|
| `hello` | A title and a button in a 1024×768 window | Startup, memory, idle cost, disk footprint |
| `list` | 10,000 rows (swatch, title, subtitle), scrolled by the app at 2,400 px/s based on elapsed time | Scrolling a big list |
| `animation` | 1,000 boxes translated and rotated over time, plus 200 labels whose text changes every frame | Per-frame updates, transforms, text |
| `settings` | A window like crownos-settings: a 240 px sidebar of 8 pages with icons, and a page of 8 titled cards of 5 rows (title, description, and a switch, slider, value or chevron). The page scrolls at 1,200 px/s and the next page opens every 2 s, at the top | A realistic app: a long page of controls, cards and icons, and page switches |

Each app uses its toolkit's idiomatic tools:

| | List | Per-frame driver | Rotation |
|---|---|---|---|
| CrownUI | `lazy_list` + `ScrollHandle::scroll_to_offset` | `cx.request_animation_frame` into one signal; boxes `bind` `translate`/`rotate` | `.rotate()` |
| GPUI 0.2 | `uniform_list` + scroll handle offset | `window.request_animation_frame()`, full re-render (immediate mode) | `svg().with_transformation(..)` (GPUI cannot rotate divs) |
| Qt 6 Quick | `ListView` (`contentY`) | `FrameAnimation`, property bindings | `rotation` |
| Electron / Tauri | 10,000 plain DOM rows, `scrollTop` | `requestAnimationFrame`, `style.transform` / `textContent` | CSS `rotate()` |

In `settings` each app builds the page the way an app in that toolkit would. CrownUI uses the kit, as crownos-settings does: `sidebar`, `sidebar_item`, `section` cards, `divider`, `toggle`, `slider` and Lucide `icon`s in a `scroll_view`, with the Inter font, and opens pages with `keyed`. GPUI uses divs and `svg()` icons in an `overflow_y_scroll` div. Qt uses `Flickable`, `Repeater`s and Qt Quick Controls `Switch` and `Slider`. The web apps use DOM elements, `<img>` icons and `<input type="range">`. The rules (titles, which control each row gets, the clock) are in `apps/shared/settings.rs`, mirrored by `apps/web/settings.js` and `apps/qt/Settings.qml`; the icons are in `apps/web/icons`.

Electron and Tauri share `apps/web`, so the gap between them is the runtime: Chromium vs WebKitGTK. CrownUI is built the way CrownOS ships its apps: `default-features = false, features = ["wayland", "impeller", "static", "tokio"]`, with Impeller linked statically and without the in-app MCP server or AT-SPI bridge. The crownui app builds against the crownui checkout it sits in, so a run measures the working tree.

## How it measures
- **Frames.** Every toolkit draws through `libwayland-client`, so each app runs with `WAYLAND_DEBUG=client`. The harness reads the protocol trace from stderr and timestamps each line as it arrives. A frame is a `wl_surface.commit` that applies a newly attached buffer. Synchronized subsurfaces count with their parent. The trace also carries each frame callback's compositor timestamp, which gives the rate the compositor offered frames at: its *pace*, which can be lower than the output mode, for example on battery. Long frames are measured against that pace. Apps that pace with `wp_fifo` instead of frame callbacks use the session's pace. Chromium sets its own frame timer from the output mode, so it can commit more frames than the compositor shows.
- **Startup.** Time from `spawn()` to the first frame. Warm caches; the first round of runs is a discarded warm-up.
- **Memory.** PSS, USS and RSS from `/proc/<pid>/smaps_rollup`, summed over the app's whole process tree. Electron and Tauri run several processes. PSS divides shared pages fairly between the processes that map them.
- **GPU memory.** `drm-total-vram` and `drm-total-gtt` from `/proc/<pid>/fdinfo`, counted once per DRM client.
- **Idle cost.** Over the measured window of `hello`: CPU time (`utime + stime`) and context switches of every thread, which count wakeups and preemptions.
- **Work per frame.** `perf stat -e instructions:u,cycles:u,task-clock`, inherited by every child process. Counting starts disabled and is switched on and off through a control FIFO, so startup and shutdown are left out. Needs `perf_event_paranoid <= 2`. CPU ms per frame depends on the clock the `powersave` governor picks, which drops when work is split over several lightly loaded cores; user-space cycles per frame do not, so the tables show both.
- **Footprint.** The app's own files (Electron counts its whole runtime directory) and the `ldd` closure of shared libraries, which other apps may share.

Fairness rules the harness enforces:
- Every app runs on native Wayland. `DISPLAY` is removed, so a toolkit that would fall back to XWayland fails instead.
- Every app uses the Mesa driver of the display GPU (`VK_DRIVER_FILES`, `__EGL_VENDOR_LIBRARY_FILENAMES`), so none gets the discrete GPU or pays for a cross-GPU copy.
- Every app is a release build.
- Runs go round-robin across frameworks.

## Layout
- `harness/`: the `crownbench` binary. `wayland_trace.rs` counts frames, `session.rs` runs an app and reads its trace, `measure.rs` measures a run, and `sample/` holds the `/proc` samplers.
- `apps/<framework>/`: one app per framework. Each is its own Cargo package with its own lockfile, sharing `target/`.
- `apps/web/`: the scenarios shared by Electron and Tauri, and the settings icons every app draws. `apps/shared/`: the config reader, layout formulas and settings rules shared by the CrownUI and GPUI apps (`#[path]` modules).
- `apps/gpui/uncapped.patch`: the change to gpui that `--uncapped` builds with.

The GPUI app pins `libc` to 0.2.189. Later versions removed `ENOATTR`, which `xattr` 0.2, a dependency of gpui 0.2.2, needs.
