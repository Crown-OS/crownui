# popover

`popover` shows content below an anchor while a `Signal<bool>` is true. The content sits in the top layer, so it paints above everything else in the window and escapes the clipping of its ancestors. Escape closes it.

Usage:

```rust
use crownui::prelude::*;

fn sort_menu(cx: &mut Cx) -> impl View + use<> {
    let open = cx.signal(false);
    let order = cx.signal("Name");

    popover(
        button(bind(move |rt| format!("Sort: {}", order.get(rt))))
            .on_click(move |cx, _| open.update(cx, |open| *open = !*open)),
        open,
        move || {
            vstack((
                button("Name").on_click(move |cx, _| {
                    order.set(cx, "Name");
                    open.set(cx, false);
                }),
                button("Date").on_click(move |cx, _| {
                    order.set(cx, "Date");
                    open.set(cx, false);
                }),
            ))
            .p(4.0)
            .gap(2.0)
            .rounded(8.0)
            .bg(WHITE)
            .border(1.0)
            .border_color(rgb(0xe5e7eb))
        },
    )
}
```

Signature:

```rust
pub fn popover<A, F, C>(anchor: A, open: Signal<bool>, content: F) -> Element<PopoverKind<A, F>>
where
    A: View,
    F: FnMut() -> C + 'static,
    C: View;
```

## Parameters

| Name | Type | Description |
|---|---|---|
| `anchor` | `impl View` | The view the popover hangs from. Usually a button that toggles `open` in its `on_click`. Required. |
| `open` | `Signal<bool>` | Whether the content is shown. The popover sets it to `false` on Escape. Required. |
| `content` | `impl FnMut() -> impl View + 'static` | Builds the content each time the popover opens. Required. |

## Returns

| Type | Description |
|---|---|
| `Element<PopoverKind<A, F>>` | A relatively positioned element that holds the anchor and the overlay. Style it and add listeners like any element. |

## Behavior

- Position. The content's top-left corner sits at the anchor's left edge, 4 logical pixels below the anchor's bottom edge. It follows the anchor's size as it changes.
- Layering. The content is in the top layer. It paints and receives the pointer above every other node, outside its ancestors' clips, so an `overflow_hidden` card or a scroll view does not cut it off. It still lives inside the window's surface.
- Lifetime. The content is built when `open` turns true and unmounted when it turns false, like `show`. Signals created inside the content start fresh each time it opens.
- Semantics. The content is wrapped in a `vstack` with the `Dialog` role and a z-index of 100.
- Escape. A key down of Escape that reaches the popover, from focus on the anchor or inside the content, sets `open` to false and stops propagation. Escape pressed while focus is elsewhere does nothing.

## Examples

### Close on an outside click

The popover does not watch clicks outside itself. Close it from a container that covers the screen, and keep presses on the anchor and the content from reaching that container:

```rust
let open = cx.signal(false);

let menu = popover(
    button("Options")
        .on_pointer_down(|_, event| event.stop_propagation())
        .on_click(move |cx, _| open.update(cx, |open| *open = !*open)),
    open,
    move || options_panel(open).on_pointer_down(|_, event| event.stop_propagation()),
);

vstack((toolbar(menu), body))
    .w_full()
    .h_full()
    .on_pointer_down(move |cx, _| {
        if open.get(cx) {
            open.set(cx, false);
        }
    })
```

The top layer content is still a descendant of the popover, so its events bubble through the tree like any other. That is why the content and the anchor stop propagation.

### Focus the content when it opens

Keyboard users need focus inside the content for Escape and Tab to work there. Use an autofocus wrapper from the [events guide](../events.md#focus-from-code) on the first control:

```rust
popover(anchor, open, move || {
    vstack((Autofocus(text_field(query, "Filter")), results(query)))
})
```

### Animate the opening

The content mounts on open, so an `enter` style animates its appearance:

```rust
popover(anchor, open, move || {
    panel()
        .transition(Spring::snappy())
        .enter(|style| style.opacity(0.0).translate_y(-4.0))
})
```

## When to use the kit instead

`popover` is the primitive. The kit builds finished overlays on it and on the top layer: `kit::popup_menu`, `kit::select`, `kit::context_menu`, `kit::context_menu_at` and `kit::tooltip`. They handle keyboard navigation, theming and closing for you. See [kit overlays](../kit-overlays.md).

## Why it works this way

Building content on open keeps a closed popover free: no nodes, no layout and no signals until someone opens it. The cost is that state inside the content resets on every open. Keep anything that must survive, such as a filter query, in a signal created outside the content closure, as `query` is above.

The top layer solves clipping without portals. The content stays in its place in the tree, so it reads the same signals and bubbles events to the same ancestors, but it paints and hit-tests as if it were at the root. The cost is that it does not flip or shift to stay inside the window. Place the anchor where there is room below it, or use a kit overlay.

## See also

- [Kit overlays](../kit-overlays.md): menus, selects, context menus and tooltips.
- [show](show.md): the conditional mounting popover uses.
- [Events](../events.md): Escape handling, focus and bubbling.
