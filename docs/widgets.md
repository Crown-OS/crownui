# Widgets

The `kit` module holds the themed CrownOS widgets. They are built from the [components](components.md) and `canvas`, and every color comes from one `Theme`. They match the look and the springs of the CrownOS desktop.

```rust,ignore
use crownui::kit::{button, ButtonVariant};

button("Save").variant(ButtonVariant::Primary)
```

Options go on the element before listeners. Every widget also accepts the normal modifiers.

## Controls

| Builder | What it does |
|---|---|
| `button(label)` | A pill button. Variants are primary, secondary, tinted, ghost and destructive, with sizes and a leading icon. |
| `icon_button(icon, label)` | A round button with only an icon and an accessible label. |
| `toggle(on)` | A switch with an optional caption. |
| `slider(value)` | A continuous or stepped slider bound to a `Signal<f32>`. |
| `radio_group(options, selected)` | A vertical list of radio buttons. |
| `choice_list(choices, selected)` | Large selectable rows with a title, subtitle and icon. |
| `toggle_group(items, selected)` | A segmented control with text or icon items. |
| `select(options, selected)` | A dropdown that opens a panel of options. |
| `popup_menu(icon, label, entries)` | A trigger button that opens a menu of entries. |
| `context_menu(items)` | A menu panel with `menu_item`, `menu_header`, `menu_separator` and `menu_submenu` rows. |
| `context_menu_at(position, items)` | Opens a `context_menu` at a pointer position above everything, flipping at the window edges and closing on a press outside, Escape or picking a row. |
| `tooltip(label, content)` | Shows a small label under (or above) `content` after the pointer rests on it for half a second. |

## Inputs

| Builder | What it does |
|---|---|
| `text_input(value)` | A text field with placeholder, secret, read only, clear button and pill shapes. |
| `search(query)` | A search field with suggestions. |
| `keybind_input(value)` | Records a keyboard shortcut such as Super and Space. |
| `calendar(today)` | A six-week month grid that pages between months. |

## Feedback

| Builder | What it does |
|---|---|
| `status(label)` | A colored dot with a label, toned as success, warning, danger or neutral. |
| `badge(label, color)` | A small pill label with an optional icon. |
| `callout(tone, title)` | A toned message box with an optional body. |
| `progress(fraction)` | A horizontal progress bar. |
| `spinner()` | An indeterminate spinner. |
| `toast(message)` and `toast_host(content)` | A notification pill, and the host that shows `use_toasts(cx).show(cx, ..)` messages bottom-center and dismisses each after its duration. |
| `step_bar(total, current)` | Dots that show the current step of a flow. |
| `segmented_bar(segments)` | A stacked bar, for example disk usage by category. |
| `tag_chip(label, dot)` | A 22px pill naming a tag, with a 6px dot in the tag's color. |

## Containers

| Builder | What it does |
|---|---|
| `card(content)` | A raised surface with rounded corners. |
| `dialog(open, content)` | A modal sheet in the window: a scrim and a centered card that traps focus and closes on Escape or a press on the scrim. |
| `section(title, children)` | A titled group of rows. |
| `setting_row(label, control)` | A label on the left and a control on the right. |
| `pane(content)` | A surface with configurable fill, border and per-corner radii. |
| `soft_shadow(content, radius)` | Wraps content in a soft, diffuse shadow. |
| `divider()` | A themed separator with color and height options. |
| `icon(body)` | A themed stroke icon from the bundled `icons` set. |

## Shell and layouts

| Builder | What it does |
|---|---|
| `quick_panel(rows)` | A control center panel. Rows are `panel_header`, `panel_slider`, `panel_section`, `panel_item`, `panel_action`, `panel_readout` and `panel_separator`. |
| `sidebar(menus, content)` | An app frame with a collapsible sidebar of items, groups and sections. |
| `sidebar_with_header(menus, header, content)` | `sidebar` with a toolbar row above the content card. |
| `sidebar_search(query)` | A search bar for that toolbar, centered between two `spacer()`s; takes every `search` option. |
| `sidebar_item(label, icon, selected)` | A sidebar row; `.trailing_text("3")` adds a muted count and `.trailing(view)` a status dot or an eject button. |
| `path_bar(segments, on_select)` | A breadcrumb of `PathSegment`s with chevrons; the last segment is the current folder, leading segments fold into an ellipsis when short of room, and `.on_segment_drag_over` / `.on_segment_drop` make each segment a drop target. |
| `wizard(steps, current, page, footer)` | A step-by-step flow with a rail of steps. |
| `hero_wizard(hero, step, content, action)` | A wizard with a large hero area, used by the installer. |
| `slide_transition(index, pane)` | Slides between panes when the index changes. |

## Theming

- `provide_configured_theme(cx)` builds the theme from the desktop's `appearance.ron` and `theme.ron` and follows edits while the app runs.
- `provide_theme(cx, theme)` sets a fixed theme for a subtree, for example `Theme::light()` or `Theme::dark()`.
- `use_theme(cx)` returns the current theme so your own widgets can read the same colors.

Register the bundled Inter font so text matches the rest of the desktop:

```rust,ignore
AppConfig::new("App").font(kit::inter_font().to_vec(), Some(kit::INTER_FAMILY))
```

## Previews

Every widget has preview entries. List and render them without opening a window:

```sh
cargo xtask preview --list
cargo xtask preview button_primary --png button.png
```

Run `cargo run -p crownui --example kit_gallery` to see the widgets live.
