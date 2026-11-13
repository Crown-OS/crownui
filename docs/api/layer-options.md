# LayerOptions

```rust
use crownui::prelude::*;

fn main() -> Result<(), crownui::RunError> {
    let bar = LayerOptions::new("example-bar", Anchor::TOP | Anchor::LEFT | Anchor::RIGHT)
        .height(LayerExtent::Fixed(36))
        .exclusive_zone(36);
    crownui::run(
        component(|_cx| hstack((text("Mon 9:41"),)).px(12.0).items_center()),
        AppConfig::new("Bar").clear_color(TRANSPARENT).layer(bar),
    )
}
```

Signature:

```rust
pub struct LayerOptions {
    pub namespace: String,
    pub layer: Layer,
    pub anchor: Anchor,
    pub width: LayerExtent,
    pub height: LayerExtent,
    pub exclusive_zone: i32,
    pub margin: LayerMargin,
    pub keyboard: KeyboardInteractivity,
}

impl LayerOptions {
    pub fn new(namespace: impl Into<String>, anchor: Anchor) -> Self
}
```

A layer surface is a window that belongs to the desktop shell instead of to the window manager's
normal stacking. Bars, docks, launchers, notification strips, lock screens and wallpapers are layer
surfaces. You turn any CrownUI window into one by passing `LayerOptions` to
[`AppConfig::layer`](app-config.md). The rest of the app does not change: the same
views, signals and listeners run inside it.

Layer surfaces need a compositor with the layer-shell protocol, which the CrownOS compositor has.
Read `cx.platform_capabilities()` and its `layer_shell` field to check at runtime.

`LayerOptions`, `Layer`, `Anchor`, `LayerExtent`, `LayerMargin` and `KeyboardInteractivity` are all
in `crownui::prelude`.

## Constructor

- `namespace` (`impl Into<String>`, required): What the surface is, for example `"crownbar"`. The compositor picks its rules for the surface by this name.

- `anchor` (`Anchor`, required): The output edges the surface is attached to. Combine edges with `|`.

`LayerOptions::new` starts on `Layer::Top`, with both extents set to `LayerExtent::Stretch`, an
exclusive zone of 0, no margin and `KeyboardInteractivity::None`.

## Builder methods

- `layer` (`fn layer(self, layer: Layer) -> Self`): The stacking band. See [Layer](#layer).

- `width` (`fn width(self, width: LayerExtent) -> Self`): The width along the output. See [LayerExtent](#layerextent).

- `height` (`fn height(self, height: LayerExtent) -> Self`): The height along the output.

- `exclusive_zone` (`fn exclusive_zone(self, zone: i32) -> Self`): Logical pixels reserved from the anchored edge so the compositor does not place windows under the surface. 0 reserves nothing. -1 asks to cover the space other surfaces reserved as well, which overlays and launchers use.

- `margin` (`fn margin(self, margin: LayerMargin) -> Self`): Space between the surface and the edges it is anchored to.

- `keyboard` (`fn keyboard(self, keyboard: KeyboardInteractivity) -> Self`): Whether and how the surface takes keyboard focus. See [KeyboardInteractivity](#keyboardinteractivity).

## Methods

- `requested_size` (`fn requested_size(&self, output: Option<(u32, u32)>, fallback: (u32, u32)) -> (u32, u32)`): The size CrownUI asks the compositor for, in logical pixels, with 0 on an axis the compositor fills. `output` is the output's size once known, and `fallback` stands in for it until then. The runtime calls it for you. You only need it in tests.

- `follows_output` (`fn follows_output(&self) -> bool`): Whether a change of output size changes the requested size. True when either extent is `LayerExtent::Output`.

## Layer

```rust
pub enum Layer {
    Background,
    Bottom,
    Top,
    Overlay,
}
```

| Variant | Sits | Typical use |
| --- | --- | --- |
| `Background` | under every window | wallpaper |
| `Bottom` | under windows, above the background | desktop icons |
| `Top` | above normal windows (the default) | panel, dock |
| `Overlay` | above everything, fullscreen windows included | lock screen, notification, launcher |

## Anchor

```rust
pub struct Anchor(/* private */);

impl Anchor {
    pub const NONE: Self;
    pub const TOP: Self;
    pub const BOTTOM: Self;
    pub const LEFT: Self;
    pub const RIGHT: Self;
    pub const ALL: Self;

    pub const fn contains(self, other: Self) -> bool;
    pub const fn bits(self) -> u32;
    pub const fn spans_width(self) -> bool;
    pub const fn spans_height(self) -> bool;
}

impl BitOr for Anchor { /* .. */ }
```

`Anchor::NONE` centers the surface on the output. `Anchor::ALL` covers the whole output. Combine
edges with `|`: `Anchor::TOP | Anchor::LEFT | Anchor::RIGHT` is a full-width top bar.

- `contains(other)` is true when every edge of `other` is part of this anchor.
- `bits()` returns the bitmask the protocol uses: top 1, bottom 2, left 4, right 8.
- `spans_width()` is true when both left and right are set.
- `spans_height()` is true when both top and bottom are set.

## LayerExtent

```rust
pub enum LayerExtent {
    Stretch,
    Fixed(u32),
    Output,
}
```

- `Stretch` (`variant`): The default. Fill the space between two opposite anchors. A stretched axis needs both of its edges anchored. When it is not, CrownUI asks for the matching side of `AppConfig::size` instead of the illegal 0.

- `Fixed(u32)` (`variant`): A fixed length in logical pixels.

- `Output` (`variant`): The output's whole length along this axis, without anchoring both edges. A top bar uses it for its height to keep room below itself for popups while it reserves only its own height through `exclusive_zone`.

## LayerMargin

```rust
pub struct LayerMargin {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}
```

Logical pixels from each anchored edge. It implements `Default`, so set only the sides you need.

```rust
LayerOptions::new("dock", Anchor::BOTTOM)
    .width(LayerExtent::Fixed(640))
    .height(LayerExtent::Fixed(72))
    .margin(LayerMargin { bottom: 8, ..LayerMargin::default() })
```

## KeyboardInteractivity

```rust
pub enum KeyboardInteractivity {
    None,
    Exclusive,
    OnDemand,
}
```

| Variant | Behavior |
| --- | --- |
| `None` | The default. Never focused. Keys go to the windows below. |
| `Exclusive` | Takes every key while mapped. Lock screens and launchers use it. |
| `OnDemand` | Focused like a normal window when the user clicks it. |

## Changing a layer surface at runtime

Two calls on `Cx` change a layer surface after it opens. Normal windows ignore both. Each takes
effect with the next frame.

- `cx.set_layer_height` (`fn set_layer_height(&mut self, height: LayerExtent)`): Resizes the surface along its height. The new size arrives as an ordinary resize, so your layout reacts to it like any other.

- `cx.set_layer_exclusive_zone` (`fn set_layer_exclusive_zone(&mut self, zone: i32)`): Changes how much of its anchored edge the surface reserves, e.g. after its resting height changed.

- `cx.set_keyboard_interactivity` (`fn set_keyboard_interactivity(&mut self, keyboard: KeyboardInteractivity)`): Changes when the surface takes keyboard focus.

A bar grows to the full output height only while a popup is open, then shrinks back:

```rust
fn open_popup(cx: &mut Cx, open: Signal<bool>) {
    open.set(cx, true);
    cx.set_layer_height(LayerExtent::Output);
}

fn close_popup(cx: &mut Cx, open: Signal<bool>) {
    open.set(cx, false);
    cx.set_layer_height(LayerExtent::Fixed(36));
}
```

A notification strip takes the keyboard only while a reply field is open, so it never steals keys
from the window the user is typing in:

```rust
fn reply_box(draft: Signal<String>) -> impl View {
    component(move |cx| {
        let replying = cx.signal(false);
        vstack((
            button("Reply").on_click(move |cx, _| {
                replying.set(cx, true);
                cx.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
            }),
            show(
                replying,
                move || {
                    text_field(draft, "Reply").on_submit(move |cx, _| {
                        replying.set(cx, false);
                        cx.set_keyboard_interactivity(KeyboardInteractivity::None);
                    })
                },
                || spacer(),
            ),
        ))
    })
}
```

## Why it works this way

A surface's size costs GPU memory for every pixel, whether or not anything is drawn there. That is
why `set_layer_height` exists: a bar that keeps a full-height surface for its popups keeps buffers
the size of the whole output allocated all the time. Growing only while a popup is open keeps the
idle cost to a thin strip.

Keyboard focus is the other cost. A surface that holds the keyboard takes every key the user meant
for the window below. Start with `KeyboardInteractivity::None` and switch to `OnDemand` only while a
field needs typing.

The options mirror the layer-shell protocol one to one, with one addition: `LayerExtent::Output`.
The protocol only lets a surface span an axis by anchoring both of its edges. `Output` takes the
output's whole length along an axis without anchoring both edges, so a top bar gets the full output
height for its popups while it reserves only its own height through `exclusive_zone`. Because
`Output` is a fixed size, CrownUI has to request it again when the output changes.
`follows_output` tells the runtime when to do that.

## Related

- [Windows guide](../windows.md)
- [AppConfig](app-config.md)
- [run_windows](run-windows.md)
