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
- Features: `wayland`, `impeller`, `static` (link libimpeller statically), `tokio` (`cx.spawn`),
  `mcp` (agents can drive the app through `crownmcp`), `accessibility` (AT-SPI).

Examples: `cargo run -p crownui --example counter` (also `todo`, `glass_card`, `gallery`).
