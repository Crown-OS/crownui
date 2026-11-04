# widgets

The primitive building blocks every CrownUI app and every `kit` widget is made of. Each is a
`view::Kind` plus a lowercase builder function returning `Element<…>`, so every Tailwind modifier,
DOM listener and variant applies.

| Function | Kind | Notes |
|---|---|---|
| `text(content)` | `TextKind` | content may be a literal, `Signal<String>`, `Memo` or `bind(..)` |
| `vstack(children)` / `hstack(..)` / `zstack(..)` | `StackKind` | flex column / row; zstack overlays children |
| `spacer()` | `SpacerKind` | grows to fill |
| `divider()` | `DividerKind` | 1px separator |
| `button(label)` | `ButtonKind` | role button, focusable, Enter/Space click |
| `toggle(on)` | `ToggleKind` | switch bound to a `Signal<bool>`, animated knob |
| `text_field(value)` | `TextFieldKind` | editable `Signal<String>`; fires `on_input`/`on_change`/`on_submit` |
| `scroll_view(content)` | `ScrollViewKind` | wheel/touchpad scrolling with clamped offset; `.scroll_handle(handle)` (`ScrollOptions`) lets app code read and drive it |
| `lazy_vstack(items, key, row_height, row)` | `LazyVStackKind` | virtualized list: only visible rows are mounted |
| `lazy_list(count, revision, row_height, row)` | `LazyListKind` | virtualized list addressed by index: `row(cx, index)` reads app data, nothing is cloned; bumping `revision` rebuilds the rows in view; `.content_padding(px)` (`LazyListOptions`), `.scroll_handle(handle)` |
| `lazy_grid(count, revision, cell, gap, item)` | `LazyListKind` | the same as an auto-fill grid (`repeat(auto-fill, minmax(cell.width, 1fr))`); whole rows in view plus 4 of overscan are mounted; `ScrollHandle::columns` reports the column count |
| `canvas(painter)` | `CanvasKind` | custom drawing into a `DrawList`, re-run on size/signal change |
| `image(width, height, rgba)` | `ImageKind` | straight-alpha RGBA8 pixels, freed on unmount |
| `cover_image(image, natural_size, radius)` | `CoverImageKind` | a registered image cropped to cover its box, rounded |
| `svg(source)` / `icon(source, tint)` | `SvgKind` | vector art via usvg (solid fills/strokes) |
| `popover(anchor, open, content)` | `PopoverKind` | overlay below the anchor; Escape closes |

Defaults are deliberately plain (system-neutral colors, no theme); `widgets::kit` layers the CrownOS
look on top. Adding a widget: see the recipe in the workspace `AGENTS.md`.

## `kit`: the CrownOS widgets

Ported from `crownuikit` with its geometry, springs and palette. Every color comes from one `Theme`
read through `use_theme(cx)`; `provide_theme(cx, theme)` sets it for a subtree and
`provide_configured_theme(cx)` builds it from `appearance.ron` (mode, accent) and `theme.ron`
(accent gradient and per-slot overrides) via `crownconfig`, following edits while mounted.

- Options are chained on the element before listeners: `button("Save").variant(ButtonVariant::Primary)`.
- Register the bundled font: `AppConfig::new(..).font(kit::inter_font().to_vec(), Some(kit::INTER_FAMILY))`.
- Shell building blocks: `quick_panel(rows)` with `panel_header` (`.switch(on)`), `panel_slider`, `panel_section`
  (`.disclosure()`), `panel_item` (`.badge`/`.glyph`/`.detail`/`.trailing`/`.selected`/`.chevron`/`.inert`),
  `panel_action`, `panel_readout`, `panel_separator` — the control-centre panels a status bar opens; and
  `calendar(today)`, a six-week month grid that pages itself (`CalendarDate` carries its own date math).
- File-manager pieces: `path_bar(segments, on_select)` (breadcrumb of `PathSegment`s, folds leading
  segments into an ellipsis when short of room, `.on_segment_drag_over`/`.on_segment_drop` per segment),
  `tag_chip(label, dot)` (22px tag pill), and `sidebar_item(..)` `.trailing_text(caption)` /
  `.trailing(view)` for counts, status dots and eject buttons.
- Overlays: `tooltip(label, content)`, `toast_host(content)` + `use_toasts(cx).show(cx, msg)` (`Toast`,
  auto-dismissed bottom-center), `dialog(open, content)` (scrim, focus trap, Escape) and
  `context_menu_at(position, items)` (pointer-anchored, flips at window edges); they cover the window
  through `util::window_cover`.
- `kit::previews()` lists one `Preview` per widget for `cargo xtask preview` and `crowndev`.
