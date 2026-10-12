# accessibility

Exposes the CrownUI `SemanticTree` to screen readers through
[AccessKit](https://accesskit.dev) (`accesskit` 0.25, `accesskit_unix` 0.24 → AT-SPI over D-Bus).

- `to_tree_update(&tree, focus)` / `diff_to_tree_update(&diff, &tree_after, focus)` convert a tree
  or a `SemanticDiff` into an `accesskit::TreeUpdate`. `TreeExporter` adds a window label, the
  toolkit name and a scale factor. Incremental updates carry the root plus the changed nodes;
  AccessKit drops nodes their parent no longer lists.
- `action_request_to_ui(&request)` turns an AccessKit request into `(SemanticId,
  AccessibilityAction)`: `Ui(UiAction)` for click, focus, hover, type-text and scroll (steps of
  `SCROLL_LINE_STEP` / `SCROLL_PAGE_STEP`), dedicated variants for increment, decrement, expand,
  collapse, blur, context menu and scroll-into-view.
- `AccessibilityAdapter::new(app_name, toolkit)` registers the window and returns
  `AccessibilityEvents`, which the UI thread drains each frame with `try_iter()`. On `Activated`,
  call `update(&tree, focus)`; afterwards `update_if_active(&diff, &tree, focus)` per frame.

Mapping: `Text` → `Label` (text in `value`, as AccessKit expects), `TextField`/`SearchField` →
`TextInput`/`SearchInput`, or `PasswordInput` when `secure`; secure values are never exported.
`checked` → `toggled`, `focusable` → the `Focus` action. `ScrollView` clips its children, so
AccessKit works out what is off screen on its own. `offscreen` nodes stay visible to screen
readers and get `ScrollIntoView`. Bounds are logical window pixels; `set_scale_factor` turns
them into the physical pixels AT-SPI expects by putting a transform on the root node. An
empty tree exports as a window with `SYNTHETIC_WINDOW_ID` (`u64::MAX`). The focus defaults to the root.

Structure is cleaned up before export: children that don't exist, repeat or form cycles are
dropped, and nodes the root can't reach are left out. AccessKit panics on any of these.

Threading: AccessKit calls its handlers on its own thread and needs them to be `Send`.
There are no locks in this crate. The handlers own `crossbeam-channel` endpoints, and actions
and activation changes go to the UI thread through them. Full trees built while no screen
reader is listening go into a one-slot channel. The adapter keeps its own copy of that
channel's receiver, so it can throw away an older tree before sending a newer one. The
activation handler takes whatever is in the slot and keeps it as the initial tree.
The runtime is `async-io` (the default) rather than tokio, so no tokio reactor is needed.
