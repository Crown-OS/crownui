use geometry::Size;
use layouting::{Dimension, FlexDirection};
use reactive::Signal;
use view::{
    Cx, Element, ItemLayout, Kind, NodeId, ScrollHandle, SizeProperty, StyleProp, Styled, View,
    lazy_index,
};

use crate::{
    scroll_options::ScrollOptions,
    scroll_view::{ScrollViewKind, scroll_view},
};

/// The kind behind [`lazy_list`] and [`lazy_grid`].
pub struct LazyListKind<F> {
    count: Signal<usize>,
    revision: Signal<u64>,
    layout: ItemLayout,
    handle: Option<ScrollHandle>,
    item: F,
}

impl<F, V> Kind for LazyListKind<F>
where
    F: FnMut(&mut Cx, usize) -> V + 'static,
    V: View,
{
    const NAME: &'static str = "lazy_list";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let handle = self.handle.unwrap_or_else(|| ScrollHandle::new(cx));
        let items = lazy_index(self.count, self.revision, self.layout, handle, self.item);
        let scroller: Element<ScrollViewKind<_>> =
            scroll_view(items).scroll_handle(handle).flex_1();
        cx.set_default_style(node, StyleProp::FlexDirection(FlexDirection::Column));
        cx.set_default_style(
            node,
            StyleProp::Size(SizeProperty::MinHeight, Dimension::Length(0.0)),
        );
        cx.set_children(node, scroller);
    }
}

impl<F, V> ScrollOptions for Element<LazyListKind<F>>
where
    F: FnMut(&mut Cx, usize) -> V + 'static,
    V: View,
{
    fn scroll_handle(self, handle: ScrollHandle) -> Self {
        self.configure(|kind| kind.handle = Some(handle))
    }
}

/// The options of a [`lazy_list`] or [`lazy_grid`], chained before its modifiers.
pub trait LazyListOptions: Sized {
    /// Space around the items, inside the scrolled content (CSS `padding` on the list).
    fn content_padding(self, padding: f32) -> Self;
}

impl<F, V> LazyListOptions for Element<LazyListKind<F>>
where
    F: FnMut(&mut Cx, usize) -> V + 'static,
    V: View,
{
    fn content_padding(self, padding: f32) -> Self {
        self.configure(|kind| kind.layout = kind.layout.with_padding(padding))
    }
}

/// A vertically scrolling list of `count` rows, each `row_height` tall, that mounts only the
/// rows in view (plus overscan). `row(cx, index)` builds a row from the app's data by index, so a
/// list of 100 000 files clones nothing; bump `revision` when that data changes and the rows in
/// view are built again.
pub fn lazy_list<V, F>(
    count: Signal<usize>,
    revision: Signal<u64>,
    row_height: f32,
    row: F,
) -> Element<LazyListKind<F>>
where
    V: View,
    F: FnMut(&mut Cx, usize) -> V + 'static,
{
    Element::new(LazyListKind {
        count,
        revision,
        layout: ItemLayout::rows(row_height),
        handle: None,
        item: row,
    })
}

/// Like [`lazy_list`], but lays the items out in a grid of as many columns of at least
/// `cell.width` as fit (CSS `repeat(auto-fill, minmax(cell.width, 1fr))`), rows `cell.height`
/// tall and `gap` (between columns, between rows) apart. Only the rows in view (plus overscan)
/// are mounted; [`ScrollHandle::columns`] says how many columns there are.
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
{
    Element::new(LazyListKind {
        count,
        revision,
        layout: ItemLayout::cells(cell, gap),
        handle: None,
        item,
    })
}
