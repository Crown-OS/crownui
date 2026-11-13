# Kit theme

Usage:

```rust
use crownui::{
    kit::{provide_theme, use_theme, Theme},
    prelude::*,
};

fn main() -> Result<(), crownui::RunError> {
    crownui::run(
        component(|cx| {
            provide_theme(cx, Theme::light());
            component(|cx| {
                let theme = use_theme(cx);
                text("Hello")
                    .p(16.0)
                    .bg(bind(move |runtime| theme.with(runtime, |theme| theme.surface.raised)))
            })
        }),
        AppConfig::new("Hello").size(320, 200),
    )
}
```

Signature:

```rust
pub fn provide_theme(cx: &mut Cx, theme: Theme) -> Signal<Theme>
pub fn provide_configured_theme(cx: &mut Cx) -> Signal<Theme>
pub fn use_theme(cx: &mut Cx) -> Signal<Theme>
pub fn panel_fill(theme: &Theme, opacity: f32) -> Color
pub fn panel_rim(theme: &Theme, opacity: f32) -> Color
```

Every kit widget reads its colors from the nearest provided `Theme`. These functions live in `crownui::kit`. `provide_theme` and `provide_configured_theme` put a theme in scope, `use_theme` reads it, and `panel_fill` and `panel_rim` derive the colors of a quick panel from it.

All three scope functions return a `Signal<Theme>`. Setting it restyles every kit widget that reads it, on the next frame.

## provide_theme

Provides `theme` to the current scope and everything built inside it.

- `cx` (`&mut Cx`, required): The context of the component that owns the scope.

- `theme` (`Theme`, required): The palette to provide, such as `Theme::light()`, `Theme::dark()` or a theme built with struct update syntax.

Returns `Signal<Theme>`: The provided theme. Set it to restyle the scope.

```rust
let theme = provide_theme(cx, Theme::dark());
button("Light").on_click(move |cx, _| theme.set(cx, Theme::light()))
```

## provide_configured_theme

Provides the theme that `~/.config/crownos/appearance.ron` and `theme.ron` describe, and keeps it following edits to either file while the calling component is mounted. The appearance picks the mode and the named accent. Then `theme.ron` replaces the accent if it sets one and edits individual slots for the current mode.

- `cx` (`&mut Cx`, required): The context of the component that owns the scope. The file watch stops when this component unmounts.

Returns `Signal<Theme>`: The provided theme. It changes on its own when the user edits either file.

The first load happens on the calling thread. It reads both files and writes their defaults if they are missing. After that, the theme follows edits through one `cx.spawn` task per change.

## use_theme

Returns the nearest provided theme. When no theme is provided, it provides `Theme::DEFAULT` on the current scope and returns that.

- `cx` (`&mut Cx`, required): The context of the component that reads the theme.

Returns `Signal<Theme>`: The theme in scope. Read it inside `bind`, a memo or an effect to follow changes.

```rust
let theme = use_theme(cx);
let border = bind(move |runtime| theme.with(runtime, |theme| theme.surface.border));
```

## Theme

`Theme` is the kit's complete palette, one `Copy` struct with every color a kit widget paints. Each field is a group of slots.

- `mode` (`ThemeMode`): Which palette this is, `ThemeMode::Light` or `ThemeMode::Dark`. Widgets read it only for the direction of a shadow or tint.

- `accent` (`GradientStops`): The accent gradient: toggle on state, slider fill, selected radio, focus rings and primary buttons.

- `surface` (`SurfaceColors`): The stack of backgrounds a window is built from, ordered by elevation, plus the lines between them. Slots:
  - `bg` (`Color`): The window's own background.
  - `raised` (`Color`): Anything lifted off the page: cards, popovers, menus.
  - `sunken` (`Color`): Anything recessed into it: sidebars, wells.
  - `border` (`Color`): The 1px border around a raised surface.
  - `hairline` (`Color`): The hairline between rows inside one surface.
  - `hover` (`Color`): The translucent hover wash painted over whatever is beneath.
  - `shadow` (`Color`): The drop shadow behind raised surfaces.

- `text` (`TextColors`): Foreground colors in descending emphasis. Icons get two slots because a stroked glyph reads lighter than text at the same value. Slots:
  - `primary` (`Color`): Headings and selected rows.
  - `body` (`Color`): Body copy and control labels.
  - `muted` (`Color`): Descriptions, section headers, read-only values and placeholders.
  - `disabled` (`Color`): Text that is present but not actionable.
  - `on_accent` (`Color`): Text and glyphs drawn on an accent fill.
  - `icon` (`Color`): The icon stroke matching `body`.
  - `icon_muted` (`Color`): The icon stroke matching `muted`.

- `control` (`ControlColors`): Slots shared by the hand-painted toggle and slider. Slots:
  - `track` (`Color`): A slider's unfilled track.
  - `knob` (`Color`): The toggle's knob and the slider's thumb.
  - `knob_shadow` (`Color`): The drop shadow under the knob.

- `toggle_off` (`GradientStops`): The off state of a switch track, top to bottom. It travels to `accent` as the switch turns on.

- `status` (`StatusColors`): Semantic colors. This is the one group that does not follow the accent, so a red accent never hides an error. Slots:
  - `success` (`Color`): Connected, saved, healthy.
  - `warning` (`Color`): Degraded but working.
  - `danger` (`Color`): Failed, or about to destroy something.
  - `neutral` (`Color`): Nothing in particular. It recedes next to the colored tones.
  - `danger_bg` (`Color`): The fill behind a destructive control.
  - `on_danger` (`Color`): Text and glyphs drawn on `danger_bg`.

- `popover` (`PopoverColors`): Field-like triggers (select, keybind, toggle group) and the surfaces that are not glass. Slots:
  - `bg` (`Color`): The background of an opaque popover surface.
  - `border` (`Color`): The surface and trigger border at rest.
  - `text` (`Color`): Labels and the selected value.
  - `muted_text` (`Color`): Section headers, the chevron and the placeholder.
  - `hover_bg` (`Color`): The hover background of rows on an opaque surface.
  - `trigger_bg` (`Color`): The trigger background.
  - `ring` (`GradientStops`): The focus accent. The trigger's border travels to these stops.

- `radio` (`RadioColors`): Radio groups: one 20px circle in its two states. Slots:
  - `accent` (`GradientStops`): The selected circle's ring, top to bottom.
  - `unselected_bg` (`Color`): The interior of an unselected circle.
  - `unselected_border` (`Color`): The 1px rim of an unselected circle, also the ring's start color.
  - `hole` (`Color`): The hole punched out of the selected circle's ring.
  - `text` (`Color`): The row label.

- `menu` (`MenuColors`): Menus and every frosted-glass surface: context and popup menus, select and search panels, tooltips, toasts and dialogs. Slots:
  - `bg` (`Color`): The tint laid over the blurred backdrop of the panel.
  - `border` (`Color`): The panel's glass rim.
  - `text` (`Color`): The row label.
  - `shortcut_text` (`Color`): The trailing keyboard shortcut.
  - `disabled_text` (`Color`): The label and shortcut of a disabled row.
  - `header_text` (`Color`): The small section header.
  - `separator` (`Color`): The hairline between sections.
  - `selected_bg` (`Color`): The fill behind the highlighted row.
  - `selected_border` (`Color`): The rim of the highlighted row's capsule.
  - `selected_text` (`Color`): The label, icon and shortcut of the highlighted row.

- `sidebar` (`SidebarColors`): The sidebar layout, which has three interpolated row states. Slots:
  - `bg` (`Color`): The column's own fill.
  - `separator` (`Color`): The hairline between groups of rows.
  - `text` (`Color`): An idle top-level row's label.
  - `selected_text` (`Color`): A selected row's label.
  - `subitem_text` (`Color`): An idle sub-item's label.
  - `brand_text` (`Color`): The brand row's label and icon.
  - `icon` (`Color`): An idle row's leading icon.
  - `selected_icon` (`Color`): A selected row's leading icon.
  - `hover_bg` (`Color`): The hover wash over an idle row.
  - `selected_bg` (`Color`): The selected row's raised pill fill.
  - `selected_border` (`Color`): The hairline around that pill.

- `title_bar` (`TitleBarColors`): Client-drawn title bars, matching the frame the compositor draws around other windows. Slots:
  - `bg` (`Color`): The bar's fill.
  - `control` (`Color`): A window control's disc.
  - `control_hover` (`Color`): The disc under the pointer.
  - `control_glyph` (`Color`): The glyph knocked into a disc.

To change a few slots, spread a stock palette:

```rust
let base = Theme::dark();
provide_theme(
    cx,
    Theme {
        sidebar: SidebarColors { bg: rgb(0x16161c), ..base.sidebar },
        ..base
    },
);
```

### Constructors and methods

| Item | What it does |
|---|---|
| `Theme::DEFAULT` | The palette in force before anything is loaded: dark with the purple accent. |
| `Theme::dark()` | The dark palette with the default accent. |
| `Theme::light()` | The light palette with the default accent. |
| `Theme::for_mode(mode, accent)` | The palette for a `ThemeMode` with a named `crownconfig::AccentColor`. |
| `Theme::with_stops(mode, stops)` | The palette for a `ThemeMode` with the accent given as `GradientStops`. |
| `Theme::from_settings(&settings)` | The palette a `crownconfig::ThemeSettings` describes. `provide_configured_theme` uses it. |
| `theme.token(path)` | The color behind a dotted token such as `"surface.bg"` or `"accent.start"`, or `None` for a name the palette does not have. |
| `theme.with_overrides(&overrides)` | This palette with each `"group.slot"` key of a `BTreeMap<String, String>` replaced by its `#rrggbb` or `#rrggbbaa` color. Unknown slots and malformed colors are skipped. |

`for_mode`, `with_stops`, `dark` and `light` are `const fn`, so a theme can be a `const`.

```rust
use std::collections::BTreeMap;

const BLUE: GradientStops = GradientStops::new(rgb(0x5aa2ff), rgb(0x2563eb));
let overrides = BTreeMap::from([(String::from("surface.bg"), String::from("#101014"))]);
let theme = Theme::with_stops(ThemeMode::Dark, BLUE).with_overrides(&overrides);
```

### ThemeMode

| Item | What it does |
|---|---|
| `ThemeMode::Dark` | Dark surfaces, light text. The CrownOS default. |
| `ThemeMode::Light` | Light surfaces, dark text. |
| `ThemeMode::from_dark_mode(dark_mode)` | The mode that `appearance.ron`'s `dark_mode` asks for. |
| `mode.is_dark()` | Whether this is `Dark`, for choosing the direction of a shadow or a tint. |

### GradientStops

`GradientStops` holds the two stops of every accent surface as the public fields `start` (the lighter stop, toward the top or left) and `end` (the deeper stop).

| Item | What it does |
|---|---|
| `GradientStops::new(start, end)` | Stops from `start` to `end`. |
| `GradientStops::flat(color)` | The same color at both ends: a flat fill in the shape of a gradient. |
| `stops.mix(target, progress)` | Blends stop for stop, so a surface travels between two gradients. |
| `stops.map(\|color\| ..)` | Applies a change to both stops. |
| `stops.fade(factor)` | Both stops with their alpha scaled by `factor`. |
| `stops.vertical(top, bottom)` | A top to bottom `Paint` between two y positions, in the painting node's coordinates. |
| `stops.horizontal(left, right)` | A left to right `Paint` between two x positions. |
| `kit::accent_stops(accent)` | The stops a named `AccentColor` resolves to. |
| `kit::DEFAULT_ACCENT` | The CrownOS purple, the accent before the user picks one. |

Use `vertical` or `horizontal` to paint the accent in your own `canvas`:

```rust
let theme = use_theme(cx);
canvas(move |runtime, size, list| {
    let accent = theme.with(runtime, |theme| theme.accent);
    let bounds = Rect::new(Point::ZERO, size);
    list.fill(bounds, CornerRadii::uniform(12.0), accent.vertical(0.0, size.height));
})
```

## Panel colors

`panel_fill(theme, opacity)` returns the quick panel body, the theme's `popover.bg` at `opacity`. The compositor's blur shows through it. `opacity` is clamped to `0..=1`.

`panel_rim(theme, opacity)` returns the hairline that keeps a translucent panel's edge off the wallpaper in both modes. It mixes the fill at `opacity` toward `text.primary` at a fixed alpha.

- `theme` (`&Theme`, required): The palette to derive from. Read it with `theme.with(runtime, ..)` or `theme.get(runtime)`.

- `opacity` (`f32`, required): The panel body's alpha, from 0 to 1.

Returns `Color`: the fill or rim color.

```rust
let theme = use_theme(cx);
vstack(rows)
    .rounded(16.0)
    .border(1.0)
    .bg(bind(move |runtime| theme.with(runtime, |theme| panel_fill(theme, 0.8))))
    .border_color(bind(move |runtime| theme.with(runtime, |theme| panel_rim(theme, 0.8))))
```

## Why one theme value

The kit keeps every color in one `Copy` struct behind one signal. A widget never holds its own color, so there is exactly one place to change the look of a window, and the desktop's settings reach every widget through it. The [kit overview](../kit.md) covers the tradeoffs in full.

Two details follow from that design. Each read through `theme.with(..)` subscribes the reader to the whole theme, not to one slot. Changing any slot wakes every reader, which is fine because themes change rarely. And `Theme` is plain data with `PartialEq`, so the kit's own tests build one with `Theme::for_mode(..)` for every accent and both modes and compare the rendered frames.
