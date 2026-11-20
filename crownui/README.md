# crownui

The one crate apps depend on. SwiftUI-style composition, React-style local state, Tailwind-named
styling, DOM-named events, Impeller rendering with render-thread animations.

```rust,no_run
use crownui::prelude::*;

fn main() -> Result<(), crownui::RunError> {
    crownui::run(
        component(|cx| {
            let count = cx.signal(0);
            vstack((
                text(bind(move |rt| format!("Count: {}", count.get(rt)))).text_size(24.0),
                button("+1")
                    .on_click(move |cx, _| count.update(cx, |count| *count += 1))
                    .hover(|style| style.scale(1.04))
                    .transition(Spring::snappy()),
            ))
            .p(24.0)
            .gap(12.0)
        }),
        AppConfig::new("Counter"),
    )
}
```

- `prelude`: everything an app author needs (views, widgets, modifiers, signals, colors, springs).
- `ext`: extension points for component libraries (`Kind`, `DrawList`, `Listener`, the backend
  contracts) — this is what `widgets::kit` builds on.
- `run(root, config)`: Wayland + Impeller (Vulkan, GLES fallback) + taffy. `run_windows(vec![..])`
  opens several windows (e.g. a toast strip and a panel), each with its own UI. Other combinations are a
  type parameter away via `runtime::Runner<G, L>`.
- `run_surfaces(app)` (`crownui::surfaces`) runs a shell app with surfaces on some or all monitors.
  `Surfaces::placement(&Output)` returns `None` (no surface) or `Some(key)`; `Surfaces::build` makes
  the surfaces for that output, and each layer surface is put on it. Surfaces open when an output
  appears, are built again when its key changes, and close when it goes or is no longer wanted; the
  app keeps running with no surface. Surfaces are placed again once per dispatch of events in which
  an output appeared, changed or went, or the app woke the loop through the `Waker` given to
  `Surfaces::attach`: first `Surfaces::outputs(&[Output])` with every output in announcement order
  (to pick, say, the focused output else the first for a single surface), then `placement` for
  each, so keep both cheap. `Output::crown`
  (`CrownOutput`, `ShellSurfaces`) says what the CrownOS compositor wants there; it is `None` on other
  compositors. A surface the compositor closes is not opened again until its output's key changes.
  Example: `per_output`.
- Features: `wayland`, `impeller`, `static` (link libimpeller statically), `tokio` (`cx.spawn`),
  `mcp` (agents can drive the app through `crownmcp`), `accessibility` (AT-SPI), `audio` (off by
  default: microphone capture as `crownui::audio` in the published crate, and `kit::use_microphone`).

Examples: `cargo run -p crownui --example counter` (also `todo`, `glass_card`, `gallery`).
