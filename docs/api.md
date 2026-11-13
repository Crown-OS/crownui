# API overview

Most apps need one import:

```rust
use crownui::prelude::*;
```

This reference documents that import, the entry points at the crate root and the extension points in
`crownui::ext`. Each page shows a usage example and the exact signature, then lists every parameter,
builder method and return value.

## Where things live

| Path | What is in it |
| --- | --- |
| `crownui::run`, `crownui::run_windows` | Open windows and run the app. |
| `crownui::AppConfig`, `crownui::FramePacing` | Window options. Also in the prelude. |
| `crownui::RunError` | The error the entry points return. |
| `crownui::prelude` | Everything an app author needs: views, widgets, modifiers, signals, colors, springs, window and layer options. |
| `crownui::kit` | The themed CrownOS widgets and `kit::icons`, the Lucide icon set. |
| `crownui::ext` | Extension points for component libraries and other backends: `Kind`, `DrawList`, `Listener`, `StyleProp`, `Runner` and the backend traits. |

The prelude groups into a few kinds of items:

- Views and structure: `component`, `show`, `keyed`, `for_each`, `error_boundary`, `bind`,
  `View`, `ViewSeq`, `AnyView`, `Element`, `IntoProp`.
- Widgets: `vstack`, `hstack`, `zstack`, `spacer`, `divider`, `text`, `button`, `toggle`,
  `text_field`, `scroll_view`, `lazy_list`, `lazy_vstack`, `lazy_grid`, `canvas`, `image`,
  `cover_image`, `svg`, `icon`, `popover`, with `LazyListOptions`, `ScrollOptions` and
  `ScrollHandle`.
- Modifier traits and their values: `Styled`, `Listeners`, `Semantic`, `Event`, `PointerEvents`,
  `ColorScheme`.
- State: `Cx`, `Runtime`, `Signal`, `Memo`, `Store`.
- Values: `Color`, `rgb`, `rgba`, `WHITE`, `BLACK`, `TRANSPARENT`, `Point`, `Size`, `Rect`,
  `Vector`, `CornerRadii`, `CornerShape`, `Shadow`, `Spring`, `Curve`, `GridTrack`, `Role`.
- Windows: `AppConfig`, `FramePacing`, `LayerOptions`, `Layer`, `Anchor`, `LayerExtent`,
  `LayerMargin`, `KeyboardInteractivity`, `Material`, `MaterialBlur`, `MaterialShadow`,
  `Capabilities`, `CursorIcon`, `AppWindows`.
- Clipboard and drag and drop: `ClipboardData`, `DragSource`, `DragIcon`, `DragOutcome`,
  `DndAction`, `DndActions`.

## Conventions

### Builders are lowercase nouns

A view builder is a function named like a SwiftUI view: `vstack`, `text`, `button`. It returns an
`Element` you chain modifiers on.

```rust
vstack((text("Title"), button("OK")))
```

### Modifiers are Tailwind names

Style methods use Tailwind class names with literal values in logical pixels. `.p(16.0)` is
Tailwind's `p-4`. Variants take a closure: `.hover(|style| ..)`, `.dark(|style| ..)`.

```rust
text("Hello").p(16.0).rounded(8.0).hover(|style| style.opacity(0.8))
```

### Listeners are DOM names

Handlers are named after DOM events (`on_click`, `on_key_down`, `on_input`) and get
`(&mut Cx, &mut Event<T>)`. Events bubble. A handler calls `event.stop_propagation()` to stop them
and `event.prevent_default()` to cancel the default action.

```rust
button("Save").on_click(|cx, event| {
    event.stop_propagation();
    save(cx);
})
```

### Any property takes a value or a reactive source

A parameter typed `impl IntoProp<T>` takes a literal, a `Signal<T>`, a `Memo<T>` or a
`bind(move |rt| ..)`. With a literal the property is fixed. With the others it updates when its
source changes.

```rust
let width = cx.signal(320.0);
vstack(()).w(320.0);                                  // fixed
vstack(()).w(width);                                  // follows the signal
vstack(()).w(bind(move |rt| width.get(rt) * 2.0));    // follows an expression
```

### State goes through `&mut Cx`

`Cx` derefs to the reactive runtime. Signal reads and writes both take it mutably, so read into a
local first, then write.

```rust
let initial = todo.with(cx, |todo| todo.done); // read
let done = cx.signal(initial);                  // then write
```

### No macros

Every API is a real function, method or type. There is no macro in the authoring API.

## Errors

The entry points return `Result<(), RunError>`.

```rust
pub enum RunError {
    Graphics(GraphicsError),
    Platform(String),
    Thread(std::io::Error),
}
```

- `RunError::Graphics` (`GraphicsError`): The graphics backend failed to initialize.

- `RunError::Platform` (`String`): The platform failed, for example when no Wayland compositor is reachable.

- `RunError::Thread` (`std::io::Error`): The OS refused to start a thread, such as a window's render thread.

Problems that do not stop the app are logged through `tracing` with a stable code such as
`CUI0402`. The [errors](errors.md) page lists every code with its fix.

## Cargo features

Default features are `wayland`, `impeller`, `tokio`, `mcp` and `accessibility`. `static` and `audio`
are off. `run` and `run_windows` exist only with `wayland` and `impeller` on, and `cx.spawn` needs
`tokio`.

## Stability

CrownUI is pre-1.0. Under Cargo's semver rules for 0.x crates, a minor release such as
0.2 can change any API on these pages. The repository keeps a snapshot of every crate's public API
in `api/*.txt`, and `cargo xtask api --check` fails when the API changes without the snapshot being
updated, so every API change is visible in review. Check the changelog on the CrownUI website
before you upgrade.

## Reference

### Running an app

- [run](api/run.md): Open one window and run until it closes.
- [run_windows](api/run-windows.md): Open several windows, each with its own UI.
- [AppConfig](api/app-config.md): Title, size, material, decorations, MCP and frame pacing.
- [LayerOptions](api/layer-options.md): Panels, docks, overlays and wallpapers.

### State

- [Cx](api/cx.md): The context every component and handler receives.
- [cx.signal](api/signal.md): Reactive state that updates only what reads it.
- [cx.memo](api/memo.md): A derived value that recomputes when its inputs change.
- [cx.effect](api/effect.md): Run code when the signals it reads change.
- [cx.store](api/store.md): Untracked state. Reads subscribe nothing and writes notify nobody.
- [bind](api/bind.md): Turn any expression into a reactive property.
- [cx.provide](api/provide.md): Pass a value down the tree without threading it through every call.

### Structure

- [component](api/component.md): A closure that creates state and returns a view, once.
- [show](api/show.md): Mount one of two branches.
- [keyed](api/keyed.md): Rebuild a branch when a derived value changes.
- [for_each](api/for-each.md): A keyed list with a signal per row.
- [error_boundary](api/error-boundary.md): Catch a panicking component and show a fallback.

### Views

- [Stacks](api/stacks.md): `vstack`, `hstack`, `zstack`, `spacer` and `divider`.
- [text](api/text.md): Static and reactive text.
- [button](api/button.md): A pressable control.
- [toggle](api/toggle.md): A switch bound to a `Signal<bool>`.
- [text_field](api/text-field.md): Text input bound to a `Signal<String>`.
- [scroll_view](api/scroll-view.md): A scrolling container.
- [lazy_list](api/lazy-list.md): Mount only the visible rows of a long list.
- [canvas](api/canvas.md): Draw paths, shapes and text into a `DrawList`.
- [image](api/image.md): Show RGBA pixels.
- [svg](api/svg.md): Show an SVG document.
- [popover](api/popover.md): Content anchored to another element.

### Modifiers

- [Styled](api/styled.md): Every Tailwind-named style method.
- [Variants](api/variants.md): Hover, active, focus, dark, enter and exit styles.
- [Listeners](api/listeners.md): DOM-named event handlers.
- [Semantic](api/semantic.md): Labels and roles for screen readers, tests and agents.
- [Spring and Curve](api/spring-curve.md): The motion that transitions use.

### Kit, testing and extension

- [Kit theme](api/kit-theme.md): Read and provide the CrownOS theme.
- [TestApp](api/test-app.md): Run a view headless and act on it.
- [Kind](api/kind.md): Write a widget from scratch.
- [DrawList](api/draw-list.md): The drawing commands a canvas records.
