# Usage

This guide walks through a small app and the patterns you use in every CrownUI program. Everything here comes from `crownui::prelude`.

## A first app

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

`component` runs its closure once to build the view. The closure never runs again. Updates happen because `text` reads `count` inside `bind`, so only that text changes when the count changes.

## Views and children

Builders are lowercase nouns such as `vstack`, `hstack`, `zstack`, `text` and `button`. Children are tuples, `Vec`s or `Option`s. Any function that returns `impl View` is a reusable piece of UI.

```rust,ignore
fn labeled(title: &'static str, content: impl View) -> impl View {
    vstack((text(title).font_semibold(), content)).gap(6.0)
}
```

## State

- `cx.signal(value)` creates a `Signal<T>`. Read it with `get`, `with` or `with2`. Write it with `set` or `update`.
- `cx.memo(..)` derives a value and recomputes only when its inputs change.
- `bind(move |rt| ..)` turns any expression into a reactive property.

Any property accepts a literal, a `Signal`, a `Memo` or a `bind`. Because `Cx` derefs to the runtime, reads and writes both need `&mut`. Read first, then write:

```rust,ignore
let initial = todo.with(cx, |todo| todo.done);
let done = cx.signal(initial);
```

## Styling

Style methods use Tailwind class names with literal values in logical pixels.

```rust,ignore
text("Hello")
    .p(16.0)
    .rounded(8.0)
    .bg(rgb(0x3b82f6))
    .text_color(WHITE)
    .hover(|style| style.bg(rgb(0x2563eb)))
    .dark(|style| style.bg(rgb(0x1e293b)))
    .transition(Spring::snappy())
```

Variants such as `hover`, `active`, `focus`, `focus_visible`, `when_disabled` and `dark` take paint properties. `.transition(..)` animates every change in the subtree with a spring or a timing curve. The full mapping from Tailwind classes to methods is in [naming.md](naming.md).

## Events

Listeners use DOM names: `on_click`, `on_key_down`, `on_input`, `on_pointer_move`, `on_drop` and so on. Every handler receives `(&mut Cx, &mut Event<T>)`. Events bubble, and handlers can call `stop_propagation()` or `prevent_default()`.

## Lists and structure

- `show(condition, then, otherwise)` mounts one of two branches and unmounts the hidden one.
- `for_each(items, key, row)` renders a keyed list and gives each row its own signal.
- `lazy_for_each` and `lazy_vstack` mount only the visible rows.
- `keyed(..)` rebuilds a branch when a derived value changes.
- `error_boundary(..)` catches a panicking component and shows a fallback.

## Using the kit

The `kit` module adds the themed CrownOS widgets. Provide a theme once near the root and register the bundled Inter font.

```rust,ignore
use crownui::{kit::{self, provide_configured_theme, section, setting_row, toggle}, prelude::*};

crownui::run(
    component(|cx| {
        provide_configured_theme(cx);
        let wifi = cx.signal(true);
        section("Network", (setting_row("Wi-Fi", toggle(wifi)),))
    }),
    AppConfig::new("Settings").font(kit::inter_font().to_vec(), Some(kit::INTER_FAMILY)),
)
```

`provide_configured_theme` reads the desktop's light or dark mode and accent color and follows edits while the app runs. See [widgets.md](widgets.md) for the full list.

## Windows and layer surfaces

- `AppConfig::new(title).size(w, h)` opens a normal window.
- `AppConfig::layer(LayerOptions)` turns the app into a panel, dock or overlay on the desktop shell.
- `crownui::run_windows(vec![..])` opens several windows from one app, each with its own UI.

## Async work

With the default `tokio` feature, `cx.spawn(future, on_done)` runs a future off the UI thread and calls `on_done` with its result back on the UI thread.

## Testing

`crowntest` runs the real pipeline headless on a manual clock. Tests use the same queries and actions as screen readers and agents.

```rust,ignore
let mut app = TestApp::new(counter());
app.click(role(Role::Button).name("+1"));
assert_eq!(app.text_of(role(Role::Text).text("Count")), "Count: 1");
insta::assert_snapshot!(app.snapshot());
```

Snapshots are plain text that list the semantic tree, the layout and the draw commands, so a diff shows exactly what changed.

## Driving an app with an AI agent

Apps built with the `mcp` feature open a socket in `$XDG_RUNTIME_DIR/crownui`. Register the broker once:

```sh
cargo install --path crownmcp
claude mcp add crownui -- crownmcp
```

The agent can then call `list_apps`, `ui_tree`, `ui_query` and `ui_act`. Use `cx.register_tool(AppTool::new(..))` to give agents app-specific actions. From a terminal, `crowndev tree` and `crowndev act` do the same.
