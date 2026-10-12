use std::collections::HashMap;

use geometry::{Size, Vector};
use graphics::TextSystem;
use layouting::{AvailableSpace, LayoutEngine, MeasureInput, Overflow};
use slotmap::SecondaryMap;

use crate::{
    NodeId,
    node::NodeContent,
    style::TextOverrides,
    text_cache::{TextCache, TextMetrics},
    tree::{ElementTree, RemovedNode},
};

/// A layout engine plus the mapping between elements and engine nodes.
pub struct LayoutState<L: LayoutEngine> {
    engine: L,
    nodes: SecondaryMap<NodeId, L::Node>,
    elements: HashMap<L::Node, NodeId>,
}

impl<L: LayoutEngine> LayoutState<L> {
    /// Wraps an engine.
    pub fn new(engine: L) -> Self {
        Self {
            engine,
            nodes: SecondaryMap::new(),
            elements: HashMap::new(),
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

fn measure_text(metrics_at: &mut dyn FnMut(Option<f32>) -> TextMetrics, input: MeasureInput) -> Size {
    if let Some(width) = input.known_width {
        let unbounded = metrics_at(None);
        let metrics = if width + WRAP_TOLERANCE >= unbounded.max_width { unbounded } else { metrics_at(Some(width)) };
        return Size::new(width, input.known_height.unwrap_or(metrics.size.height));
    }
    let unbounded = metrics_at(None);
    let metrics = match input.available_width {
        AvailableSpace::MaxContent => unbounded,
        AvailableSpace::MinContent => metrics_at(Some(unbounded.min_width)),
        AvailableSpace::Definite(width) if width + WRAP_TOLERANCE >= unbounded.max_width => unbounded,
        AvailableSpace::Definite(width) => metrics_at(Some(width)),
    };
    Size::new(metrics.size.width, input.known_height.unwrap_or(metrics.size.height))
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
    /// Syncs dirty nodes into `state`, lays out the tree in `viewport`, and writes geometry back.
    pub(crate) fn layout<L: LayoutEngine, T: TextSystem>(
        &mut self,
        state: &mut LayoutState<L>,
        text: &mut TextCache<T>,
        viewport: Size,
        removed: &[RemovedNode],
        rtl: bool,
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
        self.sync_styles(state, rtl);
        let Some(&root_node) = state.nodes.get(root) else {
            return LayoutOutcome::default();
        };
        let tree: &ElementTree = self;
        let elements = &state.elements;
        state.engine.compute_layout(root_node, viewport, |engine_node, input| {
            let Some(element) = elements.get(&engine_node).and_then(|id| tree.get(*id)) else {
                return Size::ZERO;
            };
            match &element.content {
                NodeContent::Text(content) => {
                    let style = element.resolved_text.clone().unwrap_or_default();
                    let mut metrics_at = |width: Option<f32>| text.measure(&content.text, &style, width);
                    measure_text(&mut metrics_at, input)
                }
                NodeContent::Image(image) => match (input.known_width, input.known_height) {
                    (Some(width), Some(height)) => Size::new(width, height),
                    (Some(width), None) if image.natural_size.width > 0.0 => {
                        Size::new(width, width * image.natural_size.height / image.natural_size.width)
                    }
                    (None, Some(height)) if image.natural_size.height > 0.0 => {
                        Size::new(height * image.natural_size.width / image.natural_size.height, height)
                    }
                    _ => image.natural_size,
                },
                NodeContent::Container | NodeContent::Canvas(_) => Size::ZERO,
            }
        });
        self.write_back(state, text, root, rtl)
    }

    fn sync_styles<L: LayoutEngine>(&mut self, state: &mut LayoutState<L>, rtl: bool) {
        let ids: Vec<NodeId> = self.iter().map(|(id, _)| id).collect();
        for id in &ids {
            let needs_text = self
                .get(*id)
                .is_some_and(|element| element.dirty.measure && matches!(element.content, NodeContent::Text(_)));
            let resolved = needs_text.then(|| resolve_text_style(self, *id, rtl));
            let Some(element) = self.get_mut(*id) else {
                continue;
            };
            if let Some(resolved) = resolved {
                element.resolved_text = Some(resolved);
            }
            let measured = matches!(element.content, NodeContent::Text(_) | NodeContent::Image(_));
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
                    if element.dirty.measure && measured {
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
            let children: Vec<L::Node> = element
                .children
                .iter()
                .filter_map(|child| state.nodes.get(*child).copied())
                .collect();
            if let Some(&engine_node) = state.nodes.get(id) {
                state.engine.set_children(engine_node, &children);
            }
        }
    }

    fn write_back<L: LayoutEngine, T: TextSystem>(
        &mut self,
        state: &mut LayoutState<L>,
        text: &mut TextCache<T>,
        root: NodeId,
        rtl: bool,
    ) -> LayoutOutcome {
        let mut outcome = LayoutOutcome::default();
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
            let window_rect = layout.rect.translate(parent_origin);
            if element.geometry.rect != layout.rect || element.geometry.window_rect != window_rect {
                element.dirty.frame = true;
                outcome.frames_changed = true;
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
            if let (NodeContent::Text(content), Some(style)) = (&mut element.content, element.resolved_text.as_ref())
                && (element.dirty.measure || element.dirty.frame || content.layout.is_none())
            {
                let padding = element.style.layout.padding;
                let width = (layout.rect.size.width - padding.horizontal()).max(0.0);
                let natural = text.measure(&content.text, style, None).max_width;
                let width = if width + WRAP_TOLERANCE >= natural { natural.max(width) } else { width };
                let (id, _) = text.acquire(&content.text, style, width);
                if let Some(previous) = content.layout.replace(id) {
                    text.release(previous);
                }
                element.dirty.content = true;
                element.dirty.measure = false;
            }
            let scroll_offset = if element.style.layout.overflow == Overflow::Scroll {
                element.visual.content_offset
            } else {
                Vector::ZERO
            };
            let child_origin = window_rect.origin.to_vector() - scroll_offset;
            let own_width = Some(layout.rect.size.width);
            stack.extend(element.children.iter().map(|child| (*child, child_origin, own_width)));
        }
        outcome
    }
}

/// What a layout pass changed.
#[derive(Debug, Default)]
pub(crate) struct LayoutOutcome {
    pub(crate) frames_changed: bool,
    pub(crate) canvas_sizes: Vec<(reactive::Signal<Size>, Size)>,
}
