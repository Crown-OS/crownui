use std::hash::Hash;

use reactive::Signal;
use view::{Cx, Element, Kind, Listeners, NodeId, Styled, View, Viewport, lazy_for_each};

use crate::scroll_view::scroll_view;

/// The kind behind [`lazy_vstack`].
pub struct LazyVStackKind<T, K, F> {
    items: Signal<Vec<T>>,
    key: fn(&T) -> K,
    row_height: f32,
    row: F,
}

impl<T, K, F, V> Kind for LazyVStackKind<T, K, F>
where
    T: Clone + PartialEq + 'static,
    K: Hash + Eq + Clone + 'static,
    F: FnMut(&mut Cx, Signal<T>) -> V + 'static,
    V: View,
{
    const NAME: &'static str = "lazy_vstack";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let offset = cx.signal(0.0_f32);
        let viewport = Viewport {
            offset,
            size: cx.observe_size(node),
        };
        let list = lazy_for_each(self.items, self.key, self.row_height, viewport, self.row);
        let scroller = scroll_view(list)
            .grow(1.0)
            .on_scroll(move |cx, event| offset.set(cx, event.offset.y));
        cx.set_default_style(
            node,
            view::StyleProp::FlexDirection(layouting::FlexDirection::Column),
        );
        cx.set_children(node, scroller);
    }
}

/// A vertically scrolling list that mounts only the rows in view (plus overscan), so lists with
/// thousands of items stay cheap. Every row is `row_height` tall.
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
{
    Element::new(LazyVStackKind {
        items,
        key,
        row_height,
        row,
    })
}
