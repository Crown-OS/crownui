# scroll_view

`scroll_view` clips its content and scrolls it with the wheel or touchpad. A `ScrollHandle` lets app code read and drive the scroll position.

```rust
use crownui::prelude::*;

fn release_notes(lines: Vec<String>) -> impl View {
    scroll_view(
        vstack(lines.into_iter().map(|line| text(line)).collect::<Vec<_>>())
            .gap(6.0)
            .p(16.0),
    )
    .h(320.0)
    .label("Release notes")
}
```

## Signature

```rust
pub fn scroll_view<V: View>(content: V) -> Element<ScrollViewKind<V>>
```

## Parameters

- `content: V` (required). One view. To scroll several children, wrap them in a `vstack`.

## Returns

- `Element<ScrollViewKind<V>>`. The viewport node. It has the `ScrollView` role and `overflow-scroll`. The content inside it does not shrink, so it keeps its natural size and overflows. The element also implements `ScrollOptions`.

## Give it a bounded size

A scroll view only scrolls when it is smaller than its content. Constrain it with a fixed size, or let it fill the free room of a parent that has a size.

```rust
// A fixed height.
scroll_view(content).h(300.0)

// The rest of a window-sized column.
vstack((
    toolbar(),
    scroll_view(content).grow(1.0).min_h(0.0),
))
.h_full()
```

Without a bound, a flex item grows to fit its content, and nothing overflows.

## Scrolling behavior

- The wheel or touchpad scrolls the nearest scroll view under the pointer, on both axes, clamped to the content.
- When that scroll view is already at its edge, the scroll ends there. It does not pass to an outer scroll view.
- Every wheel or touchpad scroll that moves the content fires `on_scroll` on the viewport, and the event bubbles to its ancestors. A scroll from `ScrollHandle::scroll_to_offset` changes the handle's `offset` signal and fires no event.
- `scroll_view` draws no scroll bar.

```rust
scroll_view(content)
    .h(300.0)
    .on_scroll(move |cx, event| at_top.set(cx, event.offset.y <= 0.0))
```

`event.offset` is a `Vector` with the new content offset.

## ScrollOptions

`ScrollOptions` is the trait for options shared by `scroll_view`, `lazy_list` and `lazy_grid`. Chain it before style methods.

```rust
pub trait ScrollOptions: Sized {
    fn scroll_handle(self, handle: ScrollHandle) -> Self;
}
```

- `scroll_handle(handle)` attaches a `ScrollHandle` to the container. The handle's offset becomes the content offset, and wheel scrolling writes back to the handle.

```rust
let handle = ScrollHandle::new(cx);
scroll_view(content).scroll_handle(handle).h(300.0)
```

## ScrollHandle

A `ScrollHandle` is a `Copy` handle to one scroll container's position and size. Create it in a component, pass it to the container, and keep a copy for your handlers.

### Create

- `ScrollHandle::new(runtime: &mut Runtime) -> ScrollHandle` creates a handle at the top. Pass `cx`, which derefs to the runtime.

### Read

Each of these returns a signal. Read them in a `bind`, a `memo` or an effect to react to scrolling.

- `offset(self) -> Signal<Vector>` is how far the content is scrolled.
- `size(self) -> Signal<Size>` is the size of the visible area.
- `content_size(self) -> Signal<Size>` is the size of the scrolled content.
- `columns(self) -> Signal<usize>` is how many columns a lazy grid uses. It is 1 for a list.

### Scroll

- `scroll_to_offset(self, runtime: &mut Runtime, offset: Vector)` scrolls to `offset`, clamped inside the content.
- `scroll_to_index(self, runtime: &mut Runtime, index: usize)` scrolls a lazy list or grid so the row that holds item `index` is at the top.
- `ensure_visible(self, runtime: &mut Runtime, index: usize)` scrolls a lazy list or grid as little as needed to show item `index` whole. It does nothing when the item is already in view.
- `visible_range(self, runtime: &mut Runtime) -> Range<usize>` returns the items of a lazy list or grid whose rows are at least partly in view. It tracks the offset and size, so an effect that calls it re-runs on scroll.

`scroll_to_index`, `ensure_visible` and `visible_range` need the item layout of a `lazy_list` or `lazy_grid`. On a plain `scroll_view`, the first two do nothing and `visible_range` returns `0..0`.

### Attach

- `attach(self, cx: &mut Cx, viewport: NodeId, content: NodeId)` makes a scroll container follow the handle. `scroll_view`, `lazy_list` and `lazy_grid` call it for you. Call it yourself only when you write your own scroll container as a `Kind`.

## Examples

### Back to top

```rust
fn log_view(lines: Vec<String>) -> impl View {
    component(move |cx| {
        let handle = ScrollHandle::new(cx);
        let offset = handle.offset();
        vstack((
            show(
                bind(move |rt| offset.get(rt).y > 200.0),
                move || button("Back to top").on_click(move |cx, _| handle.scroll_to_offset(cx, Vector::ZERO)),
                || vstack(()).display_none(),
            ),
            scroll_view(vstack(lines.into_iter().map(|line| text(line)).collect::<Vec<_>>()))
                .scroll_handle(handle)
                .grow(1.0)
                .min_h(0.0),
        ))
        .h_full()
    })
}
```

### Jump to the end

```rust
button("Latest").on_click(move |cx, _| {
    let content = handle.content_size().get(cx);
    handle.scroll_to_offset(cx, Vector::new(0.0, content.height));
})
```

`scroll_to_offset` clamps the value, so asking for the full content height lands at the bottom.

### Keep a selection in view

```rust
let select = move |cx: &mut Cx, index: usize| {
    selected.set(cx, index);
    handle.ensure_visible(cx, index);
};
```

This works with a `lazy_list` or `lazy_grid` that has the handle. See [lazy_list](lazy-list.md).

## Why it works this way

The scroll position lives in signals that the handle owns. App code, effects and bindings read it the same way they read any other state, and a handle can drive a container from anywhere in the component. The lazy lists use the same signals to decide which rows to mount.

The cost is that the offset belongs to one handle and one container. Two containers that must scroll together need an effect that copies one offset into the other handle. There is no scroll chaining and no scroll bar, so a nested scroll area that reaches its end stops the gesture, and apps that need a visible position indicator draw one from `offset` and `content_size`.

## See also

- [lazy_list, lazy_grid and lazy_vstack](lazy-list.md)
- [Lists](../lists.md)
- [Layout](../layout.md)
- [Events](../events.md)
