# lazy_list

`lazy_list`, `lazy_grid` and `lazy_vstack` scroll through long lists and mount only the rows in view, plus 4 rows of overscan on each side. A list of 100,000 items holds about one screen of nodes.

```rust
use crownui::prelude::*;

fn numbers() -> impl View {
    component(|cx| {
        let count = cx.signal(100_000_usize);
        let revision = cx.signal(0_u64);
        lazy_list(count, revision, 28.0, |_cx, index| {
            text(format!("Row {index}")).px(12.0)
        })
        .flex_1()
    })
}
```

## Signatures

```rust
pub fn lazy_list<V, F>(
    count: Signal<usize>,
    revision: Signal<u64>,
    row_height: f32,
    row: F,
) -> Element<LazyListKind<F>>
where
    V: View,
    F: FnMut(&mut Cx, usize) -> V + 'static,

pub fn lazy_grid<V, F>(
    count: Signal<usize>,
    revision: Signal<u64>,
    cell: Size,
    gap: (f32, f32),
    item: F,
) -> Element<LazyListKind<F>>
where
    V: View,
    F: FnMut(&mut Cx, usize) -> V + 'static,

pub fn lazy_vstack<T, K, V, F>(
    items: Signal<Vec<T>>,
    key: fn(&T) -> K,
    row_height: f32,
    row: F,
) -> Element<LazyVStackKind<T, K, F>>
where
    T: Clone + PartialEq + 'static,
    K: Hash + Eq + Clone + 'static,
    V: View,
    F: FnMut(&mut Cx, Signal<T>) -> V + 'static,
```

All three contain their own scroll view. Do not wrap them in `scroll_view`. Give them a bounded height with `.h(..)` or `.flex_1()` inside a column that has a size.

## lazy_list

A vertical list of `count` rows, addressed by index. The row builder reads your data by index, so nothing is cloned per row.

- `count: Signal<usize>` (required). How many rows there are.
- `revision: Signal<u64>` (required). Bump it when the data behind any index changes. Every mounted row is then built again.
- `row_height: f32` (required). The height of every row in logical pixels.
- `row: F` (required). A closure `FnMut(&mut Cx, usize) -> V` that builds the row for an index.

Each row is placed absolutely at its index times `row_height`, as wide as the list and `row_height` tall. The row root also gets its index as its element key.

### Data in a signal

Keep the data in a signal and derive `count` and `revision` from it with an effect.

```rust
fn file_list(files: Signal<Vec<String>>) -> impl View {
    component(move |cx| {
        let count = cx.signal(0_usize);
        let revision = cx.signal(0_u64);
        cx.effect(move |rt| {
            let len = files.with(rt, Vec::len);
            rt.untrack(|rt| {
                count.set(rt, len);
                revision.update(rt, |revision| *revision += 1);
            });
        });
        lazy_list(count, revision, 28.0, move |cx, index| {
            let name = files.with_untracked(cx, |files| files.get(index).cloned());
            text(name.unwrap_or_default()).px(12.0).truncate()
        })
        .content_padding(8.0)
        .flex_1()
    })
}
```

The effect subscribes to `files`. Writing `count` and `revision` inside `untrack` keeps the effect from subscribing to its own outputs.

Note: Bump `revision` for inserts and removals too. Rows are mounted by index, so after an insert at the top every mounted row shows stale data until the revision changes.

## lazy_grid

Like `lazy_list`, but lays items out in as many columns as fit. It follows CSS `repeat(auto-fill, minmax(cell.width, 1fr))`: each column is at least `cell.width` wide, and spare width is shared out so the cells fill each row.

- `count: Signal<usize>` (required). How many items there are.
- `revision: Signal<u64>` (required). Bump it when the data behind any index changes.
- `cell: Size` (required). The minimum cell width and the exact cell height.
- `gap: (f32, f32)` (required). The space between columns, then between rows.
- `item: F` (required). A closure `FnMut(&mut Cx, usize) -> V` that builds the cell for an index.

```rust
const TILE: Size = Size::new(96.0, 112.0);

fn photo_grid(count: Signal<usize>, revision: Signal<u64>, handle: ScrollHandle) -> impl View {
    lazy_grid(count, revision, TILE, (8.0, 8.0), |_cx, index| {
        vstack(text(format!("Photo {index}")).text_size(12.0))
            .rounded(8.0)
            .bg(rgb(0xe2e8f0))
            .justify_end()
            .p(6.0)
    })
    .content_padding(16.0)
    .scroll_handle(handle)
    .flex_1()
}
```

`handle.columns()` tells you how many columns the grid uses right now. Use it to move a selection up or down a row with the arrow keys.

## lazy_vstack

A vertical list over a `Signal<Vec<T>>`, keyed like `for_each`, that mounts only the rows in view.

- `items: Signal<Vec<T>>` (required). The list.
- `key: fn(&T) -> K` (required). The identity of an item. Pass a closure that captures nothing.
- `row_height: f32` (required). The height of every row.
- `row: F` (required). A closure `FnMut(&mut Cx, Signal<T>) -> V` that builds a row. Each row gets its own `Signal<T>`.

```rust
#[derive(Clone, PartialEq)]
struct Message {
    id: u64,
    subject: String,
}

fn inbox(messages: Signal<Vec<Message>>) -> impl View {
    lazy_vstack(
        messages,
        |message: &Message| message.id,
        48.0,
        |cx, message| {
            let id = message.with_untracked(cx, |message| message.id);
            text(bind(move |rt| message.with(rt, |message| message.subject.clone())))
                .px(16.0)
                .truncate()
                .key(id)
        },
    )
    .flex_1()
}
```

A row that stays in the window keeps its node and state while you scroll. When an item changes, only its row's signal is set.

`lazy_vstack` does not set an element key on its rows. Add `.key(id)` on the row root, as above, so the row's semantic id stays the same while rows above it mount and unmount. Tests and agents then keep finding the same row.

`lazy_vstack` takes no `ScrollHandle` and has no `content_padding`. Use `lazy_list` when app code needs to scroll the list.

## LazyListOptions

Options for `lazy_list` and `lazy_grid`, chained before style methods.

```rust
pub trait LazyListOptions: Sized {
    fn content_padding(self, padding: f32) -> Self;
}
```

- `content_padding(padding)` adds space around the items, inside the scrolled content, like CSS `padding` on the list. The first row starts below it and the last row ends above it.

`lazy_list` and `lazy_grid` also implement `ScrollOptions`:

- `scroll_handle(handle)` attaches a `ScrollHandle`. Use it to scroll to an index, keep an item in view, or read the visible range. See [scroll_view](scroll-view.md#scrollhandle).

```rust
let handle = ScrollHandle::new(cx);
let list = lazy_list(count, revision, 32.0, row)
    .content_padding(8.0)
    .scroll_handle(handle)
    .flex_1();

let reveal = move |cx: &mut Cx, index: usize| handle.ensure_visible(cx, index);
let top = move |cx: &mut Cx| handle.scroll_to_index(cx, 0);
```

### Load what is visible

`visible_range` tracks the scroll position. An effect that reads it re-runs while the user scrolls.

```rust
cx.effect(move |rt| {
    revision.get(rt);
    let visible = handle.visible_range(rt);
    request_thumbnails(rt, visible);
});
```

`request_thumbnails` is your own function.

## lazy_index

`lazy_list` and `lazy_grid` are built on `lazy_index`, exported from `crownui::ext` with `ItemLayout` and `LazyIndexKind`. Use it when you write your own list container.

```rust
pub fn lazy_index<V, F>(
    count: Signal<usize>,
    revision: Signal<u64>,
    layout: ItemLayout,
    handle: ScrollHandle,
    item: F,
) -> Element<LazyIndexKind<F>>
where
    V: View,
    F: FnMut(&mut Cx, usize) -> V + 'static,
```

It only places items. It needs a scroll container that has the same handle attached, because it reads the visible window from the handle.

```rust
use crownui::ext::{ItemLayout, lazy_index};

let handle = ScrollHandle::new(cx);
let rows = lazy_index(count, revision, ItemLayout::rows(28.0).with_padding(8.0), handle, |_cx, index| {
    text(format!("Item {index}"))
});
scroll_view(rows).scroll_handle(handle).flex_1()
```

The content of the scroll view must be the `lazy_index` itself. Content above it, such as a header inside the same scroll view, shifts the rows away from where the handle expects them.

### ItemLayout

`ItemLayout` describes how items are placed. It is `Copy`.

Fields:

- `min_width: f32` is the narrowest a column can be. `0` gives one column as wide as the list.
- `height: f32` is the height of every item.
- `gap: (f32, f32)` is the space between columns, then between rows.
- `padding: f32` is the space around all items, inside the scrolled content.

Constructors:

- `ItemLayout::rows(height: f32) -> ItemLayout` is one full-width item per row. This is what `lazy_list` uses.
- `ItemLayout::cells(cell: Size, gap: (f32, f32)) -> ItemLayout` is a grid of cells. This is what `lazy_grid` uses.
- `with_padding(self, padding: f32) -> ItemLayout` returns the same layout with padding. This is what `content_padding` sets.

Geometry helpers, all pure functions:

- `columns(&self, width: f32) -> usize` is how many columns fit in `width`, at least one.
- `column_width(&self, width: f32, columns: usize) -> f32` is the width of each column.
- `stride(&self) -> f32` is the distance from one row's top to the next.
- `ItemLayout::row_count(count: usize, columns: usize) -> usize` is how many rows `count` items fill.
- `row_top(&self, row: usize) -> f32` is the top of a row in the scrolled content.
- `content_height(&self, rows: usize) -> f32` is the height of the scrolled content.
- `visible_rows(&self, offset: f32, extent: f32, rows: usize, overscan: usize) -> Range<usize>` is the rows that intersect the viewport, widened by `overscan` rows each way.

## lazy_list compared with lazy_vstack

- Data: `lazy_list` reads by index from your own store. `lazy_vstack` takes a `Signal<Vec<T>>`.
- Row argument: `lazy_list` passes an index. `lazy_vstack` passes a `Signal<T>` per row.
- Updates: `lazy_list` rebuilds every mounted row when `revision` changes. `lazy_vstack` updates only the row whose item changed.
- Scrolling: `lazy_list` rebuilds only when the set of mounted rows changes. `lazy_vstack` recomputes its window on every change of the scroll offset.
- Control: `lazy_list` takes a `ScrollHandle` and `content_padding`. `lazy_vstack` takes neither.
- Grids: only `lazy_grid`.

## Why they work this way

Every row has the same height, so the visible window is arithmetic: the first row is the offset divided by the row height. No row outside the window is measured or mounted. Overscan mounts 4 extra rows on each side so rows are ready before they scroll in.

The costs follow from that. Rows cannot have different heights. Text in a row must fit, so use `.truncate()` or `.line_clamp(..)`. `lazy_list` clones nothing, but a revision bump remounts every row in view, so row-local state is lost and enter animations play again. `lazy_vstack` keeps row state, but it clones the visible items on every scroll step and keeps the whole `Vec` in a signal.

The [comparison benchmark](../performance.md) scrolls a 10,000-row `lazy_list` at 2,400 px/s. With frames paced by the display, CrownUI ran 2.35 M instructions per frame against 5.23 M for GPUI, and 2.36 CPU ms per frame against 1.95 for GPUI. With pacing off, it drew 2,303 frames per second against 2,356 for GPUI, at 0.31 against 0.42 CPU ms per frame. Qt used fewer instructions than both, 0.74 M per frame in the paced run. These figures come from `benchmarks/results/2026-11-07T07-43Z.md` and `2026-11-07T08-08Z.md`.

## See also

- [Lists](../lists.md)
- [scroll_view and ScrollHandle](scroll-view.md)
- [for_each](for-each.md)
