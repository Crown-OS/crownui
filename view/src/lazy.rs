use std::{collections::HashMap, hash::Hash, ops::Range};

use layouting::{Dimension, Position};
use reactive::{ScopeId, Signal};
use semantics::Role;

use crate::{
    Cx, NodeId, View,
    element::{Element, Kind},
    style_prop::{Edges, SizeProperty, StyleProp},
};

const OVERSCAN_ROWS: usize = 4;

/// Where the visible window of a lazy list is, fed by its scroll container.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    /// Scroll offset along the list axis.
    pub offset: Signal<f32>,
    /// Size of the visible area.
    pub size: Signal<geometry::Size>,
}

/// The kind behind [`lazy_for_each`].
pub struct LazyForEachKind<T, K, F> {
    items: Signal<Vec<T>>,
    key: fn(&T) -> K,
    row_height: f32,
    viewport: Viewport,
    row: F,
}

struct LazyRow<T> {
    scope: ScopeId,
    root: NodeId,
    item: Signal<T>,
    index: usize,
}

fn visible_rows(offset: f32, extent: f32, row_height: f32, count: usize) -> Range<usize> {
    if row_height <= 0.0 || count == 0 {
        return 0..0;
    }
    let first = (offset.max(0.0) / row_height).floor() as usize;
    let last = ((offset.max(0.0) + extent.max(0.0)) / row_height).ceil() as usize;
    first.saturating_sub(OVERSCAN_ROWS).min(count)..(last + OVERSCAN_ROWS).min(count)
}

impl<T, K, F, V> Kind for LazyForEachKind<T, K, F>
where
    T: Clone + PartialEq + 'static,
    K: Hash + Eq + Clone + 'static,
    F: FnMut(&mut Cx, Signal<T>) -> V + 'static,
    V: View,
{
    const NAME: &'static str = "lazy_for_each";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let Self {
            items,
            key,
            row_height,
            viewport,
            mut row,
        } = self;
        cx.set_role(node, Role::List);
        cx.set_default_style(node, StyleProp::Position(Position::Relative));
        cx.set_default_style(node, StyleProp::Shrink(0.0));
        cx.bind_style(node, move |runtime| {
            let count = items.with(runtime, Vec::len);
            StyleProp::Size(SizeProperty::Height, Dimension::Length(count as f32 * row_height))
        });
        cx.queue_rebuild_when(node, move |runtime| {
            items.with(runtime, |_| ());
            viewport.offset.get(runtime);
            viewport.size.with(runtime, |size| size.height);
        });
        let mut rows: HashMap<K, LazyRow<T>> = HashMap::new();
        let rebuild = Box::new(move |cx: &mut Cx, node: NodeId| {
            let offset = viewport.offset.with_untracked(cx, |offset| *offset);
            let extent = viewport.size.with_untracked(cx, |size| size.height);
            let window: Vec<(usize, T)> = items.with_untracked(cx, |list| {
                visible_rows(offset, extent, row_height, list.len())
                    .map(|index| (index, list[index].clone()))
                    .collect()
            });
            let mut order = Vec::with_capacity(window.len());
            let mut next_rows = HashMap::with_capacity(window.len());
            for (index, item) in window {
                let item_key = key(&item);
                let mut entry = match rows.remove(&item_key) {
                    Some(existing) => {
                        if existing.item.with_untracked(cx, |old| *old != item) {
                            existing.item.set(cx, item);
                        }
                        existing
                    }
                    None => {
                        let (scope, root, item_signal) = cx.mount_row(item, &mut row);
                        cx.set_default_style(root, StyleProp::Position(Position::Absolute));
                        cx.set_default_style(root, StyleProp::Inset(Edges::Horizontal, 0.0));
                        cx.set_default_style(root, StyleProp::Size(SizeProperty::Height, Dimension::Length(row_height)));
                        LazyRow {
                            scope,
                            root,
                            item: item_signal,
                            index: usize::MAX,
                        }
                    }
                };
                if entry.index != index {
                    entry.index = index;
                    cx.set_default_style(entry.root, StyleProp::Inset(Edges::Top, index as f32 * row_height));
                }
                order.push(entry.root);
                next_rows.insert(item_key, entry);
            }
            for (_, stale) in rows.drain() {
                cx.unmount_row(stale.scope, stale.root);
            }
            rows = next_rows;
            cx.adopt_children(node, order);
        });
        cx.install_rebuild(node, rebuild);
    }
}

/// Like `for_each`, but only rows inside `viewport` (plus a few of overscan) are mounted; rows have
/// a fixed `row_height`. Use it through `widgets::lazy_vstack`, which wires the viewport.
pub fn lazy_for_each<T, K, V, F>(
    items: Signal<Vec<T>>,
    key: fn(&T) -> K,
    row_height: f32,
    viewport: Viewport,
    row: F,
) -> Element<LazyForEachKind<T, K, F>>
where
    T: Clone + PartialEq + 'static,
    K: Hash + Eq + Clone + 'static,
    V: View,
    F: FnMut(&mut Cx, Signal<T>) -> V + 'static,
{
    Element::new(LazyForEachKind {
        items,
        key,
        row_height,
        viewport,
        row,
    })
}

#[cfg(test)]
mod tests {
    use super::visible_rows;

    #[test]
    fn the_visible_window_includes_overscan_and_stays_in_bounds() {
        assert_eq!(visible_rows(0.0, 100.0, 20.0, 1000), 0..9);
        assert_eq!(visible_rows(1000.0, 100.0, 20.0, 1000), 46..59);
        assert_eq!(visible_rows(19_990.0, 100.0, 20.0, 1000), 995..1000);
        assert_eq!(visible_rows(0.0, 100.0, 20.0, 0), 0..0);
    }
}
