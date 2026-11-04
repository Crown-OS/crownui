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
  Clicks carry the held `modifiers`; a drop target picks an action with `event.accept_with(mime, DndAction::Move)`
  and the drop reports the settled `action`. Events bubble; handlers can `stop_propagation()` and `prevent_default()`.
- **Scrolling**: a `ScrollHandle` (Copy) attached to a scroll container exposes its `offset`, `size`, `content_size`
  and (for lazy grids) `columns` as signals, and scrolls it with `scroll_to_offset`, `scroll_to_index`, `ensure_visible`.
- **State**: `Cx` derefs to `reactive::Runtime`, so `cx.signal(0)`, `count.get(cx)`, `count.set(cx, 1)`.
  Any property accepts a literal, a `Signal`, a `Memo` or `bind(move |rt| ..)`.
- **Structure**: `component`, `show`, `keyed` (rebuilds a branch when a derived value changes), `for_each` (keyed, per-row signals), `lazy_for_each` (virtualized),
  `lazy_index` (virtualized by index over an `ItemLayout` of rows or auto-fill cells; bumping its `revision` rebuilds what is in view),
  `error_boundary`; `.enter(..)`/`.exit(..)` animate appearance and removal.
- **Platform services**: `cx.start_drag(DragSource, on_end)` (system drag and drop, the compositor drawing the icon),
  `cx.scale_factor()` (size pixel art to the device), `cx.request_animation_frame` (UI-thread animation, e.g. a spring that drives layout),
  `cx.set_idle_inhibited` (keep the system awake while visible), `cx.set_keyboard_interactivity` (a layer surface
  takes keyboard focus only while it needs it, e.g. while a reply field is open), `cx.set_layer_height` (a layer
  surface grows only while a popup needs the room; every pixel of surface costs GPU memory), `cx.set_app_icon`/`remove_app_icon`/`activate_app_icon`/`on_launch_app`/`on_app_windows`/`on_app_icon_lifted`
  (a launcher's icons, which the compositor opens and minimizes the apps' windows into; `capabilities.app_icons`),
  `cx.on_close_requested(|cx, event| event.prevent_default())`/`cx.close_window()` (keep the window open while work
  runs or ask first; without listeners it closes as before), `cx.spawn` (async; tasks spawned before the runtime installs its executor start when it does),
  `cx.register_image`/`cx.release_image`/`cx.release_image_on_cleanup`, `cx.copy_to_clipboard`/`paste_from_clipboard`
  (text) and `cx.set_clipboard(items)`/`cx.read_clipboard(mime_types, on_read)` (several MIME types, e.g. copied files),
  `cx.register_tool(AppTool::new(..))` (actions for MCP agents), `cx.observe_size(node)`.
- **Pipeline** (`Ui`): `handle_event` → `update` → `layout(&mut LayoutState<L>, &mut TextCache<T>)` →
  `take_frame()` (render ops, surface state, cursor) and `take_app_icon_requests()` → `semantic_tree()` / `perform(id, UiAction)`.

Invariants: the UI is single-threaded and owns all state; effects never touch the tree directly
(they queue commands applied after each flush); the render thread only sees `RenderOp`s. Nodes with a
platform material snap their window-space edges to the device pixel grid (`Rect::snap_to_physical`, at
`Cx::scale_factor`), so the body CrownUI paints and the material shape the compositor cuts cover the same pixels.
While the platform composites a window's one blurring material shape with its glass (`Ui::set_composited_materials`),
that body is filled square (`Visual::body_edge` is `BodyEdge::Hard`) and its edge is the compositor's alone; its
border, ring, shadow and children's clip stay rounded.

Diagnostics: CUI0001 stale handle · CUI0101/0102 query errors · CUI0103–0106 action errors ·
CUI0201/0202 invalid variant styles · CUI0301 component panic.
