# show

`show` mounts one of two branches and switches when a condition changes. The hidden branch is unmounted, so its signals, effects and tasks are freed.

```rust
use crownui::prelude::*;

fn details() -> impl View {
    component(|cx| {
        let open = cx.signal(false);
        vstack((
            button("Toggle details").on_click(move |cx, _| open.update(cx, |open| *open = !*open)),
            show(open, || text("Size: 4.2 MB"), || text("Collapsed")),
        ))
        .gap(8.0)
    })
}
```

## Signature

```rust
pub fn show<A, B, T, E>(
    condition: impl IntoProp<bool>,
    then: T,
    otherwise: E,
) -> Element<ShowKind<T, E>>
where
    A: View,
    B: View,
    T: FnMut() -> A + 'static,
    E: FnMut() -> B + 'static,
```

## Parameters

- `condition: impl IntoProp<bool>` (required). A `bool`, a `Signal<bool>`, a `Memo<bool>` or a `bind(move |rt| ..)` closure that returns `bool`.
- `then: T` (required). A closure with no arguments that builds the view shown while the condition is true.
- `otherwise: E` (required). A closure with no arguments that builds the view shown while the condition is false.

Both branch closures are `FnMut`. `show` calls one of them every time the condition flips, so capture `Copy` handles such as signals, or clone what you need inside.

## Returns

- `Element<ShowKind<T, E>>`. A wrapper node that holds the current branch. It takes every style, event and semantic method. Styles apply to the wrapper, not to the branch.

## Behavior

- On build, `show` reads the condition once without tracking and mounts the matching branch.
- When the condition's inputs change, `show` recomputes it. If the result is the same branch, nothing happens.
- When the branch flips, the old branch unmounts first. Its scope is disposed, so local signals reset the next time it shows.
- The wrapper lays its child out as a column, the default flex direction.

## Show nothing

Pass an empty stack styled `display_none` as the branch that shows nothing. While that branch is active, the wrapper collapses too, so it takes no space and adds no gap between its siblings.

```rust
vstack((
    text("Inbox").font_bold(),
    show(has_error, || text("Sync failed").text_color(rgb(0xdc2626)), || vstack(()).display_none()),
    text("3 new messages"),
))
.gap(12.0)
```

An empty `vstack(())` without `display_none` has zero height but still takes a gap slot.

## Derive the condition

Use `bind` to compute the condition from other signals. `show` only reacts when the computed `bool` changes, not on every change of the inputs.

```rust
show(
    bind(move |rt| items.with(rt, Vec::is_empty)),
    || text("No items yet").text_color(rgb(0x64748b)),
    move || for_each(items, |item: &Item| item.id, |_cx, item| item_row(item)),
)
```

Here `item_row` is your own function that takes a `Signal<Item>` and returns `impl View`. The `otherwise` closure captures only the `items` handle, which is `Copy`, so `show` can call it again on each switch.

## show compared with hidden

`.hidden(condition)` keeps a node mounted and keeps its space. It only stops painting. `show` removes the branch from the tree.

```rust
// Keeps state and layout space. Stops painting.
text_field(note, "Note").hidden(bind(move |rt| !editing.get(rt)))

// Frees the field and its state while hidden.
show(editing, move || text_field(note, "Note"), || vstack(()).display_none())
```

Use `show` when the hidden branch is expensive or owns state that must reset. Use `.hidden` for fast toggles that keep state and layout.

## Animate the switch

Put `.enter` and `.exit` on the branch roots. The removed branch stays drawn while its exit runs, and leaves the semantic tree at once.

```rust
show(
    visible,
    || text("Saved").exit(|style| style.opacity(0.0)).transition(Spring::smooth()),
    || vstack(()).display_none(),
)
```

## Why it works this way

`show` unmounts the hidden branch instead of hiding it. A closed dialog or an unused page holds no nodes, no observers and no tasks. The cost is rebuild time on every switch and the loss of branch-local state. When you need the state to survive, keep it in a signal created above the `show` and pass it in.

## See also

- [keyed](keyed.md) for more than two branches
- [Lists](../lists.md)
- [Animation](../animation.md)
