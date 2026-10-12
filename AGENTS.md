# CrownUI — guide for AI agents and humans

CrownUI is a SwiftUI-style Rust UI toolkit for CrownOS: declarative views built once, fine-grained
signals for state, Tailwind-named styling, DOM-named events, Impeller rendering on a dedicated
render thread, and a semantic tree shared by accessibility, MCP agents and tests.

## Commands
| Task | Command |
|---|---|
| Check changed crates (clippy + tests + rustfmt on changed files) | `cargo xtask check` (`--json` for machine output) |
| Enforce the crate layering | `cargo xtask layers` |
| Public API snapshots | `cargo xtask api` / `cargo xtask api --check` |
| Pending insta snapshots | `cargo xtask snapshot` |
| Recompile `impeller/shaders/*.frag` (or `--check` they are current) | `cargo xtask shaders [--check]` |
| Run an example | `cargo run -p crownui --example counter` (`todo`, `glass_card`, `gallery`, `kit_gallery`) |
| Render a kit widget | `cargo xtask preview --list`, `cargo xtask preview <name> [--png <path>]` |
| Drive a running app as an agent | `claude mcp add crownui -- crownmcp`, then `list_apps`, `ui_tree`, `ui_query`, `ui_act` |
| One crate's gates | `cargo clippy -p <crate> --all-targets -- -D warnings && cargo test -p <crate>` |

Never run `cargo fmt --all` in sibling repos (crownpositor is not fmt-clean); format the files you touch.
The `/e` volume is nearly full: prefer `CARGO_INCREMENTAL=0` for one-off builds.

## Crate map (see layers.toml; lower layers never depend on higher ones)
| Layer | Crates |
|---|---|
| foundations | `geometry` (Point/Rect/Color/Transform, `Length` for stable text), `input` (DOM-named events, `Keybind`), `reactive` (signals), `motion` (analytic springs), `inspect` (devtools snapshots), `crownconfig` (the only door to `crownos-config`: appearance, theme overrides, keybinds, file watching) |
| contracts | `windowing` (`Platform`, `PlatformSurface`, backends `wayland`/`headless`), `graphics` (`DrawList`, `GraphicsBackend`, `RecordingBackend`), `layouting` (`LayoutEngine`), `semantics` (tree, `UiAction`, queries), `gesture` (recognizers, arena, focus) |
| implementations | `impeller` (Vulkan + GLES renderer, Paragraph text), `taffy-layout`, `accessibility` (AccessKit/AT-SPI), `mcp` (in-app MCP server) |
| core | `view` (Element, Kind, modifiers, Cx, Ui pipeline), `runtime` (Runner, render thread, InlineApp) |
| components | `widgets` (primitives: text, stacks, button, toggle, text_field, scroll_view, canvas, image; and `widgets::kit`, the themed CrownOS widgets) |
| public | `crownui` (facade + `run`), `crowntest` (test harness), `crownmcp` (agent broker), `crowndev` (devtools CLI) |

Every crate's `README.md` is its crate-level doc: purpose, public API, invariants, non-goals.

## Invariants
- No `Mutex`, `RefCell`, `Rc` or `Arc<Mutex<..>>` in framework code. State lives in `reactive::Runtime`
  and is reached through `&mut Cx`; threads talk through channels only.
- Libraries use `thiserror`; binaries, examples and xtask use `anyhow`. No `unwrap`/`expect` in libraries.
- One concept per file; `mod.rs`/`lib.rs` only declare and re-export; helpers live in `src/util/<name>.rs`.
- No macros in the public authoring API (rust-analyzer and agents must see real types).
- Diagnostics carry a stable code (`CUI0001` …) indexed in `docs/errors.md`.

## Naming (canonical table: docs/naming.md)
- Builders are SwiftUI-like lowercase nouns: `vstack`, `text`, `button`.
- Style methods are Tailwind class names with literal values: `.p(16.0)`, `.rounded(8.0)`, `.items_center()`,
  variants `.hover(|s| ..)`, `.dark(|s| ..)`, `.transition(Spring::snappy())`.
- Events are DOM names: `.on_click`, `.on_key_down`, `.on_input`; handlers get `(&mut Cx, &mut Event<T>)`.
- Test and agent actions are Testing Library names: `click`, `type_text`, `press_key`, `scroll`, `drag`.

## Borrowing pattern
`Cx` derefs to the runtime, so reads and writes both need `&mut`. Read first, then write:
```rust,ignore
let initial = todo.with(cx, |todo| todo.done); // read
let done = cx.signal(initial);                  // then write
```
Use `cx.with2(a, b, ..)` or `signal.update_from(cx, other, ..)` to combine two signals.

## Recipes
### Add a widget
1. `widgets/src/<name>.rs`: a `pub struct <Name>Kind` implementing `view::Kind` (`NAME`, `mount`), and
   `pub fn <name>(..) -> Element<<Name>Kind>`.
2. In `mount`: `cx.set_role`, `cx.set_default_style(node, StyleProp::..)` for defaults, `cx.set_children`,
   `cx.add_listener(node, Listener::Click(..))`, `cx.bind_style` for reactive defaults.
3. Re-export in `widgets/src/lib.rs` and add a row to `widgets/README.md`; add it to `crownui::prelude`.
4. Test with `crowntest` (`TestApp::new(..)`, `click(role(..).name(..))`, `insta::assert_snapshot!(app.snapshot())`).

### Add a style modifier
1. Add a `StyleProp` variant in `view/src/style_prop.rs` with its `impact()` and `apply()`.
2. Add the Tailwind-named method to `Styled` in `view/src/styled.rs` and a row in `docs/naming.md`.
3. If it paints, map it in `runtime/src/render/record.rs` (and `VisualValues` if it animates).

### Add a custom-painted component (kit style)
Use `canvas(|rt, size, list| ..)` for drawing into a `DrawList`, or a `Kind` that calls `cx.set_canvas`.
Signals read inside the painter re-run it; size changes re-run it.

### Add a windowing backend
New feature-gated module under `windowing/src/<backend>/` implementing `Platform`, `WindowControl`
and `PlatformSurface`, plus a runner that drives `ApplicationHandler<P>`. Nothing above changes.

### Add a graphics backend
Implement `graphics::GraphicsBackend` (text system + renderer sharing a layout type) and
`CreateRenderer<S>` for the surfaces it supports. Translate `DrawList`/`Composition`; see `impeller`.

### Add a CrownOS protocol
XML in `crownos-protocols/protocols/`, server in `crownpositor/protocols/src/`, client in
`windowing/src/wayland/protocols/`, surfaced through `CrownEvent` and `Capabilities`. Version every interface.

### Add or migrate a kit widget (`widgets/src/kit/`)
Rebuild it from `widgets` primitives + `canvas`, read every color from `use_theme(cx)` (never a literal), size
and animate it like the crownuikit original (springs in `util/spring.rs`), add a `previews/<name>.rs` entry and a
`widgets/tests/kit_<name>.rs` test. Theme colors come from `crownconfig` (`appearance.ron`, `theme.ron`): nothing
outside `crownconfig` imports `crownos-config`.
