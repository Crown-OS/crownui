use std::ops::Range;

use geometry::{Size, Vector};
use layouting::{Dimension, Position};
use reactive::{ScopeId, Signal};
use rustc_hash::FxHashMap;
use semantics::Role;

use crate::{
    Cx, NodeId, View,
    cx::Command,
    element::{Element, Kind},
    item_layout::ItemLayout,
    scroll_handle::{ItemPlacement, ScrollHandle},
    style_prop::{Edges, SizeProperty, StyleProp},
};

const OVERSCAN_ROWS: usize = 4;

/// The kind behind [`lazy_index`].
pub struct LazyIndexKind<F> {
    count: Signal<usize>,
    revision: Signal<u64>,
    layout: ItemLayout,
    handle: ScrollHandle,
    item: F,
}

struct MountedItem {
    scope: ScopeId,
    root: NodeId,
    frame: (f32, f32, f32),
}

/// What decides which items are mounted and where; a rebuild runs only when it changes.
#[derive(Debug, Clone, PartialEq)]
struct Window {
    items: Range<usize>,
    columns: usize,
    column_width: f32,
    revision: u64,
}

impl Window {
    fn measure(layout: &ItemLayout, count: usize, revision: u64, offset: f32, size: Size) -> Self {
        let columns = layout.columns(size.width);
        let rows = ItemLayout::row_count(count, columns);
        let visible = layout.visible_rows(offset, size.height, rows, OVERSCAN_ROWS);
        Self {
            items: visible.start * columns..(visible.end * columns).min(count),
            columns,
            column_width: layout.column_width(size.width, columns),
            revision,
        }
    }

    fn frame_of(&self, layout: &ItemLayout, index: usize) -> (f32, f32, f32) {
        let column = index % self.columns;
        let left = layout.padding + column as f32 * (self.column_width + layout.gap.0);
        (
            left,
            layout.row_top(index / self.columns),
            self.column_width,
        )
    }
}

impl<F, V> Kind for LazyIndexKind<F>
where
    F: FnMut(&mut Cx, usize) -> V + 'static,
    V: View,
{
    const NAME: &'static str = "lazy_index";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let Self {
            count,
            revision,
            layout,
            handle,
            mut item,
        } = self;
        cx.set_role(node, Role::List);
        cx.set_default_style(node, StyleProp::Position(Position::Relative));
        cx.set_default_style(node, StyleProp::Shrink(0.0));
        let size = handle.size();
        cx.bind_style(node, move |runtime| {
            let columns = layout.columns(size.with(runtime, |size| size.width));
            let rows = ItemLayout::row_count(count.get(runtime), columns);
            StyleProp::Size(
                SizeProperty::Height,
                Dimension::Length(layout.content_height(rows)),
            )
        });
        watch_window(cx, node, layout, count, revision, handle);
        let mut mounted: FxHashMap<usize, MountedItem> = FxHashMap::default();
        let mut built_revision = None;
        let rebuild = Box::new(move |cx: &mut Cx, node: NodeId| {
            let total = count.with_untracked(cx, |count| *count);
            let generation = revision.with_untracked(cx, |revision| *revision);
            let viewport = size.with_untracked(cx, |size| *size);
            let offset = clamp_offset(cx, handle, &layout, total, viewport);
            let window = Window::measure(&layout, total, generation, offset, viewport);
            handle.place_items(
                cx,
                ItemPlacement {
                    layout,
                    columns: window.columns,
                    count: total,
                },
            );
            if built_revision.replace(generation) != Some(generation) {
                for (_, stale) in mounted.drain() {
                    cx.unmount_branch(stale.scope, stale.root);
                }
            }
            let stale: Vec<usize> = mounted
                .keys()
                .copied()
                .filter(|index| !window.items.contains(index))
                .collect();
            for index in stale {
                if let Some(gone) = mounted.remove(&index) {
                    cx.unmount_branch(gone.scope, gone.root);
                }
            }
            let mut order = Vec::with_capacity(window.items.len());
            for index in window.items.clone() {
                let entry = mounted
                    .entry(index)
                    .or_insert_with(|| mount_item(cx, index, &mut item, layout.height));
                let frame = window.frame_of(&layout, index);
                if entry.frame != frame {
                    entry.frame = frame;
                    place(cx, entry.root, frame);
                }
                order.push(entry.root);
            }
            cx.adopt_children(node, order);
        });
        cx.install_rebuild(node, rebuild);
    }
}

/// Queues a rebuild of `node` whenever the mounted window would change; scrolling within the
/// overscan only re-measures.
fn watch_window(
    cx: &mut Cx,
    node: NodeId,
    layout: ItemLayout,
    count: Signal<usize>,
    revision: Signal<u64>,
    handle: ScrollHandle,
) {
    let commands = cx.commands;
    let mut last = None;
    cx.effect(move |runtime| {
        let window = Window::measure(
            &layout,
            count.get(runtime),
            revision.get(runtime),
            handle.offset().get(runtime).y,
            handle.size().get(runtime),
        );
        let primed = last.is_some();
        if last.as_ref() != Some(&window) {
            last = Some(window);
            if primed {
                commands.update(runtime, |queue| queue.push(Command::Rebuild(node)));
            }
        }
    });
}

/// Pulls the handle back inside the content when it shrank under the scroll position (a short
/// folder after a long one), returning the offset to lay out with.
fn clamp_offset(
    cx: &mut Cx,
    handle: ScrollHandle,
    layout: &ItemLayout,
    count: usize,
    viewport: Size,
) -> f32 {
    let offset = handle.offset().with_untracked(cx, |offset| *offset);
    let rows = ItemLayout::row_count(count, layout.columns(viewport.width));
    let limit = (layout.content_height(rows) - viewport.height).max(0.0);
    if offset.y > limit && viewport.height > 0.0 {
        handle.offset().set(cx, Vector::new(offset.x, limit));
        return limit;
    }
    offset.y
}

fn mount_item<V: View>(
    cx: &mut Cx,
    index: usize,
    item: &mut impl FnMut(&mut Cx, usize) -> V,
    height: f32,
) -> MountedItem {
    let (scope, root) = cx.mount_branch(|cx| item(cx, index).build(cx));
    cx.set_default_style(root, StyleProp::Position(Position::Absolute));
    cx.set_default_style(
        root,
        StyleProp::Size(SizeProperty::Height, Dimension::Length(height)),
    );
    if let Some(element) = cx.tree.get_mut(root) {
        element.key = Some(index as u64);
    }
    MountedItem {
        scope,
        root,
        frame: (f32::NAN, f32::NAN, f32::NAN),
    }
}

fn place(cx: &mut Cx, root: NodeId, (left, top, width): (f32, f32, f32)) {
    cx.set_default_style(root, StyleProp::Inset(Edges::Left, left));
    cx.set_default_style(root, StyleProp::Inset(Edges::Top, top));
    cx.set_default_style(
        root,
        StyleProp::Size(SizeProperty::Width, Dimension::Length(width)),
    );
}

/// A virtual list over `count` items addressed by index: only the items whose rows are inside
/// `handle`'s viewport (plus a few rows of overscan) are mounted, each built by `item(cx, index)`
/// and placed by `layout`. Bump `revision` when the data behind the indices changes and every
/// mounted item is built again, so nothing is cloned per change. Use it through
/// `widgets::lazy_list` or `widgets::lazy_grid`, which wire the scroll container.
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
{
    Element::new(LazyIndexKind {
        count,
        revision,
        layout,
        handle,
        item,
    })
}
