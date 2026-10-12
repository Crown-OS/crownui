# view

The declarative core of CrownUI: what app and widget authors write against.

- **Views** (`View`, `ViewSeq`, `AnyView`) are built once into an element tree. Children are tuples,
  `Vec`s or `Option`s.
- **Elements** (`Element<K: Kind>`) carry an ordered list of `Modifier`s. `Kind` is the extension
  point for widgets: `mount` sets role, content (`set_text`, `set_canvas`), default style and children.
- **Styling** (`Styled`) uses Tailwind names with literal values: `.p(16.0).bg(rgb(0x3b82f6)).rounded(8.0)`.
  Variants: `.hover(|s| ..)`, `.active`, `.focus`, `.focus_visible`, `.when_disabled`, `.dark` — values may be
  literals or bound (`.hover(|s| s.bg(bind(move |rt| theme.with(rt, |t| t.accent))))`).
  `.transition(Spring::snappy())` animates changes in the subtree; `.enter(|s| s.opacity(0.0))` animates appearance.
- **Events** (`Listeners`) use DOM names: `on_click`, `on_pointer_down`, `on_key_down`, `on_input` …;
  drops from any app arrive as HTML `on_drag_over` (call `event.accept(mime)`), `on_drag_leave`, `on_drop`.
  Events bubble; handlers can `stop_propagation()` and `prevent_default()`.
- **State**: `Cx` derefs to `reactive::Runtime`, so `cx.signal(0)`, `count.get(cx)`, `count.set(cx, 1)`.
  Any property accepts a literal, a `Signal`, a `Memo` or `bind(move |rt| ..)`.
- **Structure**: `component`, `show`, `keyed` (rebuilds a branch when a derived value changes), `for_each` (keyed, per-row signals), `lazy_for_each` (virtualized),
  `error_boundary`; `.enter(..)`/`.exit(..)` animate appearance and removal.
- **Platform services**: `cx.start_drag(DragSource, on_end)` (system drag and drop, the compositor drawing the icon),
  `cx.scale_factor()` (size pixel art to the device), `cx.request_animation_frame` (UI-thread animation, e.g. a spring that drives layout),
  `cx.set_idle_inhibited` (keep the system awake while visible), `cx.set_app_icon`/`remove_app_icon`/`activate_app_icon`/`on_launch_app`
  (a launcher's icons, which the compositor opens and minimizes the apps' windows into; `capabilities.app_icons`), `cx.spawn` (async; tasks spawned before the runtime installs its executor start when it does),
  `cx.register_image`/`cx.release_image`/`cx.release_image_on_cleanup`, `cx.copy_to_clipboard`/`paste_from_clipboard`,
  `cx.register_tool(AppTool::new(..))` (actions for MCP agents), `cx.observe_size(node)`.
- **Pipeline** (`Ui`): `handle_event` → `update` → `layout(&mut LayoutState<L>, &mut TextCache<T>)` →
  `take_frame()` (render ops, surface state, cursor) and `take_app_icon_requests()` → `semantic_tree()` / `perform(id, UiAction)`.

Invariants: the UI is single-threaded and owns all state; effects never touch the tree directly
(they queue commands applied after each flush); the render thread only sees `RenderOp`s.

Diagnostics: CUI0001 stale handle · CUI0101/0102 query errors · CUI0103–0106 action errors ·
CUI0201/0202 invalid variant styles · CUI0301 component panic.
