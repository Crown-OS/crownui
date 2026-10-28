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
| `step_bar(total, current)` | Dots that show the current step of a flow. |
| `segmented_bar(segments)` | A stacked bar, for example disk usage by category. |

## Containers

| Builder | What it does |
|---|---|
| `card(content)` | A raised surface with rounded corners. |
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
