use std::borrow::Cow;

use geometry::{Size, Vector};
use graphics::{TextStyle, TextSystem};
use layouting::{AvailableSpace, LayoutEngine, MeasureInput, MeasuredAxis, Overflow};
use rustc_hash::FxHashMap;
use slotmap::SecondaryMap;

use crate::{
    NodeId,
    node::{ElementNode, NodeContent},
    style::TextOverrides,
    text_cache::{TextCache, TextMetrics},
    tree::{ElementTree, RemovedNode},
};

/// A layout engine plus the mapping between elements and engine nodes.
pub struct LayoutState<L: LayoutEngine> {
    engine: L,
    nodes: SecondaryMap<NodeId, L::Node>,
    elements: FxHashMap<L::Node, NodeId>,
}

impl<L: LayoutEngine> LayoutState<L> {
    /// Wraps an engine.
    pub fn new(engine: L) -> Self {
        Self {
            engine,
            nodes: SecondaryMap::new(),
            elements: FxHashMap::default(),
        }
    }

    /// The wrapped engine.
    pub fn engine(&self) -> &L {
        &self.engine
    }
}

/// Widths within this many logical pixels of the text's natural width do not wrap it; it absorbs
/// float noise between measuring and laying out.
const WRAP_TOLERANCE: f32 = 0.5;

fn measure_text(
    metrics_at: &mut dyn FnMut(Option<f32>) -> TextMetrics,
    input: MeasureInput,
) -> Size {
    if let Some(width) = input.known_width {
        let unbounded = metrics_at(None);
        let metrics = if width + WRAP_TOLERANCE >= unbounded.max_width {
            unbounded
        } else {
            metrics_at(Some(width))
        };
        return Size::new(width, input.known_height.unwrap_or(metrics.size.height));
    }
    let unbounded = metrics_at(None);
    let metrics = match input.available_width {
        AvailableSpace::MaxContent => unbounded,
        AvailableSpace::MinContent if input.axis == MeasuredAxis::Width => {
            return Size::new(unbounded.min_width, unbounded.size.height);
        }
        AvailableSpace::MinContent => metrics_at(Some(unbounded.min_width)),
        AvailableSpace::Definite(width) if width + WRAP_TOLERANCE >= unbounded.max_width => {
            unbounded
        }
        AvailableSpace::Definite(width) => metrics_at(Some(width)),
    };
    Size::new(
        metrics.size.width,
        input.known_height.unwrap_or(metrics.size.height),
    )
}

/// Whether a text node whose content changed measures exactly as before under any constraint, so
/// the layout engine may keep its cached answers: the old and new text are single unbreakable runs
/// of the same size, and the node was laid out wide enough for it. Records the new metrics.
fn keeps_its_size<T: TextSystem>(element: &mut ElementNode, text: &mut TextCache<T>) -> bool {
    let (NodeContent::Text(content), Some(style)) =
        (&mut element.content, element.resolved_text.as_ref())
    else {
        return false;
    };
    let metrics = text.measure(&content.text, style, None);
    let previous = content.measured.replace(metrics);
    let box_width = element.geometry.rect.size.width - element.style.layout.padding.horizontal();
    previous == Some(metrics)
        && metrics.min_width == metrics.max_width
        && box_width + WRAP_TOLERANCE >= metrics.max_width
}

fn resolve_text_style(tree: &ElementTree, node: NodeId, rtl: bool) -> graphics::TextStyle {
    let mut overrides = TextOverrides::default();
    for ancestor in tree.ancestors(node) {
        if let Some(element) = tree.get(ancestor) {
            overrides.inherit_from(&element.text);
            if ancestor == node {
                overrides.max_lines = element.text.max_lines;
                overrides.ellipsis = element.text.ellipsis;
            }
        }
    }
    overrides.resolve(rtl)
}

impl ElementTree {
    /// Syncs dirty nodes into `state`, lays out the tree in `viewport`, and writes geometry back;
    /// material nodes snap to the device pixel grid at `scale`.
    pub(crate) fn layout<L: LayoutEngine, T: TextSystem>(
        &mut self,
        state: &mut LayoutState<L>,
        text: &mut TextCache<T>,
        viewport: Size,
        removed: &[RemovedNode],
        rtl: bool,
        scale: f32,
    ) -> LayoutOutcome {
        for gone in removed {
            if let Some(engine_node) = state.nodes.remove(gone.id) {
                state.elements.remove(&engine_node);
                state.engine.remove_node(engine_node);
            }
            if let Some(layout) = gone.text_layout {
                text.release(layout);
            }
        }
        let Some(root) = self.root() else {
            return LayoutOutcome::default();
        };
        let visited = self.sync_styles(state, text, rtl);
        let Some(&root_node) = state.nodes.get(root) else {
            return LayoutOutcome::default();
        };
        let tree: &ElementTree = self;
        let elements = &state.elements;
        state
            .engine
            .compute_layout(root_node, viewport, |engine_node, input| {
                let Some(element) = elements.get(&engine_node).and_then(|id| tree.get(*id)) else {
                    return Size::ZERO;
                };
                match &element.content {
                    NodeContent::Text(content) => {
                        let style = element
                            .resolved_text
                            .as_ref()
                            .map_or_else(|| Cow::Owned(TextStyle::default()), Cow::Borrowed);
                        let mut metrics_at =
                            |width: Option<f32>| text.measure(&content.text, &style, width);
                        measure_text(&mut metrics_at, input)
                    }
                    NodeContent::Image(image) => match (input.known_width, input.known_height) {
                        (Some(width), Some(height)) => Size::new(width, height),
                        (Some(width), None) if image.natural_size.width > 0.0 => Size::new(
                            width,
                            width * image.natural_size.height / image.natural_size.width,
                        ),
                        (None, Some(height)) if image.natural_size.height > 0.0 => Size::new(
                            height * image.natural_size.width / image.natural_size.height,
                            height,
                        ),
                        _ => image.natural_size,
                    },
                    NodeContent::Container | NodeContent::Canvas(_) => Size::ZERO,
                }
            });
        let mut outcome = self.write_back(state, text, root, rtl, scale);
        outcome.visited += visited;
        outcome
    }

    /// Syncs the queued nodes into the engine; returns how many it visited.
    fn sync_styles<L: LayoutEngine, T: TextSystem>(
        &mut self,
        state: &mut LayoutState<L>,
        text: &mut TextCache<T>,
        rtl: bool,
    ) -> u32 {
        let ids = std::mem::take(&mut self.queues.layout);
        let mut visited = 0;
        for id in &ids {
            let needs_text = self.get(*id).is_some_and(|element| {
                element.dirty.measure && matches!(element.content, NodeContent::Text(_))
            });
            let resolved = needs_text.then(|| resolve_text_style(self, *id, rtl));
            let Some(element) = self.get_mut(*id) else {
                continue;
            };
            visited += 1;
            element.queued.layout = false;
            if let Some(resolved) = resolved {
                element.resolved_text = Some(resolved);
            }
            let measured = matches!(
                element.content,
                NodeContent::Text(_) | NodeContent::Image(_)
            );
            match state.nodes.get(*id) {
                None => {
                    let engine_node = state.engine.create_node(&element.style.layout, measured);
                    state.nodes.insert(*id, engine_node);
                    state.elements.insert(engine_node, *id);
                    element.dirty.children = true;
                }
                Some(&engine_node) => {
                    if element.dirty.style {
                        state.engine.set_style(engine_node, &element.style.layout);
                    }
                    if element.dirty.measure && measured && !keeps_its_size(element, text) {
                        state.engine.mark_dirty(engine_node);
                    }
                }
            }
            element.dirty.style = false;
            if !matches!(element.content, NodeContent::Text(_)) {
                element.dirty.measure = false;
            }
        }
        for id in ids {
            let Some(element) = self.get(id) else {
                continue;
            };
            if !element.dirty.children {
                continue;
            }
            visited += 1;
            let children: Vec<L::Node> = element
                .children
                .iter()
                .filter_map(|child| state.nodes.get(*child).copied())
                .collect();
            if let Some(&engine_node) = state.nodes.get(id) {
                state.engine.set_children(engine_node, &children);
            }
        }
        visited
    }

    fn write_back<L: LayoutEngine, T: TextSystem>(
        &mut self,
        state: &mut LayoutState<L>,
        text: &mut TextCache<T>,
        root: NodeId,
        rtl: bool,
        scale: f32,
    ) -> LayoutOutcome {
        let mut outcome = LayoutOutcome::default();
        let mut render_marks = Vec::new();
        let mut visited = Vec::new();
        let mut stack = vec![(root, Vector::ZERO, None::<f32>)];
        while let Some((id, parent_origin, parent_width)) = stack.pop() {
            let Some(&engine_node) = state.nodes.get(id) else {
                continue;
            };
            let mut layout = state.engine.layout(engine_node);
            if let (true, Some(width)) = (rtl, parent_width) {
                layout.rect = layouting::mirror_for_rtl(layout.rect, width);
            }
            let Some(element) = self.get_mut(id) else {
                continue;
            };
            outcome.visited += 1;
            visited.push(id);
            let mut window_rect = layout.rect.translate(parent_origin);
            if element.style.material.is_some() {
                window_rect = window_rect.snap_to_physical(scale);
                layout.rect = window_rect.translate(-parent_origin);
            }
            let moved =
                element.geometry.rect != layout.rect || element.geometry.window_rect != window_rect;
            let resized = element.geometry.rect.size != layout.rect.size;
            let descend = moved
                || element.queued.needs_visit
                || element.geometry.content_size != layout.content_size;
            element.queued.needs_visit = false;
            if moved {
                element.dirty.frame = true;
                outcome.frames_changed = true;
                outcome.moved.push(id);
            }
            element.geometry.rect = layout.rect;
            element.geometry.window_rect = window_rect;
            element.geometry.content_size = layout.content_size;
            if let NodeContent::Canvas(canvas) = &element.content {
                outcome.canvas_sizes.push((canvas.size, layout.rect.size));
            }
            if let Some(signal) = element.size_signal {
                outcome.canvas_sizes.push((signal, layout.rect.size));
            }
            let mut reshaped = false;
            if let (NodeContent::Text(content), Some(style)) =
                (&mut element.content, element.resolved_text.as_ref())
                && (element.dirty.measure || resized || content.layout.is_none())
            {
                let padding = element.style.layout.padding;
                let width = (layout.rect.size.width - padding.horizontal()).max(0.0);
                let natural = text.measure(&content.text, style, None).max_width;
                let width = if width + WRAP_TOLERANCE < natural {
                    Some(width)
                } else if style.places_lines_by_width() {
                    Some(natural.max(width))
                } else {
                    None
                };
                let (layout_id, _) = text.acquire(&content.text, style, width);
                if content.layout == Some(layout_id) {
                    text.release(layout_id);
                } else {
                    if let Some(previous) = content.layout.replace(layout_id) {
                        text.release(previous);
                    }
                    element.dirty.content = true;
                    reshaped = true;
                }
                element.dirty.measure = false;
            }
            if moved || reshaped {
                render_marks.push(id);
            }
            if !descend {
                continue;
            }
            let scroll_offset = if element.style.layout.overflow == Overflow::Scroll {
                element.visual.content_offset
            } else {
                Vector::ZERO
            };
            let child_origin = window_rect.origin.to_vector() - scroll_offset;
            let own_width = Some(layout.rect.size.width);
            stack.extend(
                element
                    .children
                    .iter()
                    .map(|child| (*child, child_origin, own_width)),
            );
        }
        for id in render_marks {
            self.mark_render(id);
        }
        self.refresh_subtree_rects(&visited);
        outcome
    }

    /// Recomputes the hit-test bounds of `visited` nodes, children before parents: `visited` is
    /// in pre-order, so walking it backwards sees every child before its parent.
    fn refresh_subtree_rects(&mut self, visited: &[NodeId]) {
        for id in visited.iter().rev() {
            let Some(element) = self.get(*id) else {
                continue;
            };
            let mut bounds = element.geometry.window_rect;
            if !element.visual.clip {
                for child in &element.children {
                    if let Some(child) = self.get(*child) {
                        bounds = bounds.union(child.subtree_rect);
                    }
                }
            }
            if let Some(element) = self.get_mut(*id) {
                element.subtree_rect = bounds;
            }
        }
    }
}

/// What a layout pass changed.
#[derive(Debug, Default)]
pub(crate) struct LayoutOutcome {
    pub(crate) frames_changed: bool,
    /// Nodes whose window rect changed, for semantic bounds to follow.
    pub(crate) moved: Vec<NodeId>,
    pub(crate) canvas_sizes: Vec<(reactive::Signal<Size>, Size)>,
    pub(crate) visited: u32,
}

#[cfg(test)]
mod tests {
    use graphics::RecordingTextSystem;
    use reactive::ScopeId;

    use super::*;
    use crate::node::TextContent;

    fn label(text: &str, width: f32) -> ElementNode {
        let mut node = ElementNode::new(None, ScopeId::default(), "text");
        node.content = NodeContent::Text(TextContent {
            text: text.into(),
            layout: None,
            measured: None,
        });
        node.resolved_text = Some(TextStyle::default());
        node.geometry.rect.size.width = width;
        node
    }

    fn retext(node: &mut ElementNode, text: &str) {
        if let NodeContent::Text(content) = &mut node.content {
            content.text = text.into();
        }
    }

    #[test]
    fn a_counter_ticking_to_a_number_of_the_same_width_keeps_its_layout() {
        let mut cache = TextCache::new(RecordingTextSystem);
        let mut counter = label("1234", 100.0);
        assert!(!keeps_its_size(&mut counter, &mut cache));
        retext(&mut counter, "5678");
        assert!(keeps_its_size(&mut counter, &mut cache));
        retext(&mut counter, "56789");
        assert!(!keeps_its_size(&mut counter, &mut cache));
    }

    #[test]
    fn text_that_can_wrap_or_does_not_fit_is_laid_out_again() {
        let mut cache = TextCache::new(RecordingTextSystem);
        let mut words = label("ab cd", 100.0);
        keeps_its_size(&mut words, &mut cache);
        retext(&mut words, "ef gh");
        assert!(!keeps_its_size(&mut words, &mut cache));
        let mut squeezed = label("1234", 1.0);
        keeps_its_size(&mut squeezed, &mut cache);
        retext(&mut squeezed, "5678");
        assert!(!keeps_its_size(&mut squeezed, &mut cache));
    }
}
