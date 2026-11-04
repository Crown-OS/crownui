use std::ops::Range;

use geometry::{Size, Vector};
use reactive::{Runtime, Signal};

use crate::{Cx, NodeId, handler::Listener, item_layout::ItemLayout, style_prop::StyleProp};

/// Where an index-based lazy list placed its items, so a handle can find one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ItemPlacement {
    pub(crate) layout: ItemLayout,
    pub(crate) columns: usize,
    pub(crate) count: usize,
}

impl ItemPlacement {
    fn content_height(&self) -> f32 {
        let rows = ItemLayout::row_count(self.count, self.columns);
        self.layout.content_height(rows)
    }

    fn row_of(&self, index: usize) -> usize {
        index.min(self.count.saturating_sub(1)) / self.columns.max(1)
    }
}

/// Reads and drives one scroll container from app code: create it with
/// [`ScrollHandle::new`], hand it to `scroll_view`, `lazy_list` or `lazy_grid`, then scroll with
/// [`scroll_to_offset`](Self::scroll_to_offset), [`scroll_to_index`](Self::scroll_to_index) or
/// [`ensure_visible`](Self::ensure_visible) and read which part is in view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollHandle {
    offset: Signal<Vector>,
    size: Signal<Size>,
    content: Signal<Size>,
    columns: Signal<usize>,
    placement: Signal<Option<ItemPlacement>>,
}

impl ScrollHandle {
    /// A handle for one scroll container, at the top.
    pub fn new(runtime: &mut Runtime) -> Self {
        Self {
            offset: runtime.signal(Vector::ZERO),
            size: runtime.signal(Size::ZERO),
            content: runtime.signal(Size::ZERO),
            columns: runtime.signal(1),
            placement: runtime.signal(None),
        }
    }

    /// How far the content is scrolled.
    pub fn offset(self) -> Signal<Vector> {
        self.offset
    }

    /// Size of the visible area.
    pub fn size(self) -> Signal<Size> {
        self.size
    }

    /// Size of the scrolled content.
    pub fn content_size(self) -> Signal<Size> {
        self.content
    }

    /// How many columns a lazy grid currently lays its items out in (1 for a list), e.g. to move
    /// a selection a row up with the arrow keys.
    pub fn columns(self) -> Signal<usize> {
        self.columns
    }

    /// Scrolls to `offset`, kept inside the content.
    pub fn scroll_to_offset(self, runtime: &mut Runtime, offset: Vector) {
        let size = self.size.with_untracked(runtime, |size| *size);
        let content = self.extent(runtime);
        let clamped = Vector::new(
            offset.x.clamp(0.0, (content.width - size.width).max(0.0)),
            offset.y.clamp(0.0, (content.height - size.height).max(0.0)),
        );
        if self
            .offset
            .with_untracked(runtime, |current| *current != clamped)
        {
            self.offset.set(runtime, clamped);
        }
    }

    /// Scrolls a lazy list or grid so the row holding item `index` is at the top.
    pub fn scroll_to_index(self, runtime: &mut Runtime, index: usize) {
        let Some(placement) = self
            .placement
            .with_untracked(runtime, |placement| *placement)
        else {
            return;
        };
        let top = placement.layout.row_top(placement.row_of(index)) - placement.layout.padding;
        let x = self.offset.with_untracked(runtime, |offset| offset.x);
        self.scroll_to_offset(runtime, Vector::new(x, top));
    }

    /// Scrolls a lazy list or grid as little as needed to show item `index` whole.
    pub fn ensure_visible(self, runtime: &mut Runtime, index: usize) {
        let Some(placement) = self
            .placement
            .with_untracked(runtime, |placement| *placement)
        else {
            return;
        };
        let layout = placement.layout;
        let top = layout.row_top(placement.row_of(index)) - layout.padding;
        let bottom = top + layout.height + 2.0 * layout.padding;
        let offset = self.offset.with_untracked(runtime, |offset| *offset);
        let height = self.size.with_untracked(runtime, |size| size.height);
        let y = if top < offset.y {
            top
        } else if bottom > offset.y + height {
            bottom - height
        } else {
            return;
        };
        self.scroll_to_offset(runtime, Vector::new(offset.x, y));
    }

    /// The items of a lazy list or grid whose rows are at least partly in view, e.g. to load
    /// thumbnails for the visible page only.
    pub fn visible_range(self, runtime: &mut Runtime) -> Range<usize> {
        let Some(placement) = self.placement.get(runtime) else {
            return 0..0;
        };
        let offset = self.offset.get(runtime).y;
        let extent = self.size.get(runtime).height;
        let rows = ItemLayout::row_count(placement.count, placement.columns);
        let visible = placement.layout.visible_rows(offset, extent, rows, 0);
        let columns = placement.columns.max(1);
        visible.start * columns..(visible.end * columns).min(placement.count)
    }

    /// Makes `viewport`, a scroll container around `content`, follow this handle: its content
    /// offset is the handle's offset, and wheel scrolling updates it. Called by scroll containers.
    pub fn attach(self, cx: &mut Cx, viewport: NodeId, content: NodeId) {
        cx.follow_size(viewport, self.size);
        cx.follow_size(content, self.content);
        let offset = self.offset;
        cx.bind_style(viewport, move |runtime| {
            StyleProp::ContentOffset(offset.get(runtime))
        });
        cx.add_listener(
            viewport,
            Listener::Scroll(Box::new(move |cx, event| {
                if offset.with_untracked(cx, |current| *current != event.offset) {
                    offset.set(cx, event.offset);
                }
            })),
        );
    }

    pub(crate) fn place_items(self, runtime: &mut Runtime, placement: ItemPlacement) {
        if self
            .placement
            .with_untracked(runtime, |current| *current != Some(placement))
        {
            self.placement.set(runtime, Some(placement));
        }
        if self
            .columns
            .with_untracked(runtime, |columns| *columns != placement.columns)
        {
            self.columns.set(runtime, placement.columns);
        }
    }

    fn extent(self, runtime: &mut Runtime) -> Size {
        let observed = self.content.with_untracked(runtime, |content| *content);
        match self
            .placement
            .with_untracked(runtime, |placement| *placement)
        {
            Some(placement) => Size::new(observed.width, placement.content_height()),
            None => observed,
        }
    }
}

impl Cx {
    /// Makes `signal` follow `node`'s laid-out size, like [`Cx::observe_size`] with a signal the
    /// caller already holds.
    fn follow_size(&mut self, node: NodeId, signal: Signal<Size>) {
        match self.tree.get(node).and_then(|element| element.size_signal) {
            Some(existing) if existing != signal => {
                self.effect(move |runtime| {
                    let size = existing.get(runtime);
                    if signal.with_untracked(runtime, |current| *current != size) {
                        signal.set(runtime, size);
                    }
                });
            }
            Some(_) => {}
            None => {
                if let Some(element) = self.tree.get_mut(node) {
                    element.size_signal = Some(signal);
                }
            }
        }
    }
}
