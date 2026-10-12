use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use geometry::{Color, Rect};
use graphics::{BoundaryId, Composition, DrawList, Renderer};
use view::{NodeKey, RenderContent, RenderOp, Visual};

use super::{Animated, VisualValues, compose, record};

pub(crate) struct RenderNode {
    pub(crate) children: Vec<NodeKey>,
    pub(crate) parent: Option<NodeKey>,
    pub(crate) exiting: bool,
    pub(crate) frame: Animated<Rect>,
    pub(crate) values: Animated<VisualValues>,
    pub(crate) visual: Visual,
    pub(crate) content: RenderContent,
    pub(crate) needs_record: bool,
    pub(crate) recorded_values: Option<VisualValues>,
    pub(crate) recorded_size: Option<geometry::Size>,
    pub(crate) has_list: bool,
}

impl RenderNode {
    fn new() -> Self {
        Self {
            children: Vec::new(),
            parent: None,
            exiting: false,
            frame: Animated::new(Rect::ZERO),
            values: Animated::new(VisualValues::from_visual(&Visual::default())),
            visual: Visual::default(),
            content: RenderContent::None,
            needs_record: true,
            recorded_values: None,
            recorded_size: None,
            has_list: false,
        }
    }
}

/// Counters for one rendered frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RenderStats {
    /// Boundaries re-recorded this frame.
    pub recorded: usize,
    /// Nodes in the tree.
    pub nodes: usize,
    /// Whether something is still animating.
    pub animating: bool,
}

/// The render thread's retained tree: applies [`RenderOp`]s, runs animations, records changed
/// boundaries into the renderer and composes frames.
pub struct RenderTree {
    pub(crate) nodes: HashMap<NodeKey, RenderNode>,
    pub(crate) root: Option<NodeKey>,
    pending_text_releases: Vec<graphics::TextLayoutId>,
    pending_image_releases: Vec<graphics::ImageId>,
    clear_color: Color,
    frame: u64,
}

impl Default for RenderTree {
    fn default() -> Self {
        Self::new(geometry::TRANSPARENT)
    }
}

impl RenderTree {
    /// An empty tree that clears frames to `clear_color`.
    pub fn new(clear_color: Color) -> Self {
        Self {
            nodes: HashMap::new(),
            root: None,
            pending_text_releases: Vec::new(),
            pending_image_releases: Vec::new(),
            clear_color,
            frame: 0,
        }
    }

    /// Applies ops from the UI thread at time `now`.
    pub fn apply<R: Renderer>(
        &mut self,
        ops: impl IntoIterator<Item = RenderOp>,
        now: Duration,
        renderer: &mut R,
    ) {
        for op in ops {
            match op {
                RenderOp::SetRoot(key) => self.root = Some(key),
                RenderOp::SetChildren(key, mut children) => {
                    for child in &children {
                        self.nodes
                            .entry(*child)
                            .or_insert_with(RenderNode::new)
                            .parent = Some(key);
                    }
                    let lingering: Vec<NodeKey> =
                        self.nodes.get(&key).map_or_else(Vec::new, |node| {
                            node.children
                                .iter()
                                .copied()
                                .filter(|child| {
                                    !children.contains(child)
                                        && self.nodes.get(child).is_some_and(|child| child.exiting)
                                })
                                .collect()
                        });
                    children.extend(lingering);
                    self.nodes
                        .entry(key)
                        .or_insert_with(RenderNode::new)
                        .children = children;
                }
                RenderOp::Remove(key, exit) => match exit {
                    Some((visual, transaction)) if self.nodes.contains_key(&key) => {
                        if let Some(node) = self.nodes.get_mut(&key) {
                            node.values.set(
                                VisualValues::from_visual(&visual),
                                Some(transaction),
                                now,
                            );
                            node.visual = Visual {
                                hidden: false,
                                ..visual
                            };
                            node.exiting = true;
                            node.needs_record = true;
                        }
                    }
                    _ => self.remove_subtree(key, renderer),
                },
                RenderOp::Frame(key, rect, transaction) => {
                    let node = self.nodes.entry(key).or_insert_with(RenderNode::new);
                    let size_changed = node.frame.target().size != rect.size;
                    node.frame.set(rect, transaction, now);
                    node.needs_record |= size_changed;
                }
                RenderOp::Visual(key, visual, transaction) => {
                    let node = self.nodes.entry(key).or_insert_with(RenderNode::new);
                    node.values
                        .set(VisualValues::from_visual(&visual), transaction, now);
                    node.visual = visual;
                    node.needs_record = true;
                }
                RenderOp::Content(key, content) => {
                    let node = self.nodes.entry(key).or_insert_with(RenderNode::new);
                    node.content = content;
                    node.needs_record = true;
                }
            }
        }
    }

    /// Records every node again on the next frame, e.g. after the renderer's corner shape changed.
    pub(crate) fn record_all_again(&mut self) {
        for node in self.nodes.values_mut() {
            node.needs_record = true;
        }
    }

    /// Releases `ids` from the renderer once no node draws them anymore.
    pub fn release_text_when_unused(
        &mut self,
        ids: impl IntoIterator<Item = graphics::TextLayoutId>,
    ) {
        self.pending_text_releases.extend(ids);
    }

    /// Frees `ids` from the renderer once no node draws them anymore.
    pub fn release_images_when_unused(&mut self, ids: impl IntoIterator<Item = graphics::ImageId>) {
        self.pending_image_releases.extend(ids);
    }

    pub(crate) fn release_when_unused(&mut self, releases: crate::batch::Releases) {
        self.release_text_when_unused(releases.text);
        self.release_images_when_unused(releases.images);
    }

    fn flush_releases<R: Renderer>(&mut self, renderer: &mut R) {
        if self.pending_text_releases.is_empty() && self.pending_image_releases.is_empty() {
            return;
        }
        let mut text_in_use = HashSet::new();
        let mut images_in_use = HashSet::new();
        for node in self.nodes.values() {
            match &node.content {
                RenderContent::Text { layout, .. } => {
                    text_in_use.insert(*layout);
                }
                RenderContent::Image(image) => {
                    images_in_use.insert(*image);
                }
                RenderContent::Canvas(list) => images_in_use.extend(list.images()),
                RenderContent::None => {}
            }
        }
        self.pending_text_releases.retain(|id| {
            let keep = text_in_use.contains(id);
            if !keep {
                renderer.release_text(*id);
            }
            keep
        });
        self.pending_image_releases.retain(|id| {
            let keep = images_in_use.contains(id);
            if !keep {
                renderer.release_image(*id);
            }
            keep
        });
    }

    fn remove_subtree<R: Renderer>(&mut self, key: NodeKey, renderer: &mut R) {
        let mut stack = vec![key];
        while let Some(current) = stack.pop() {
            let Some(node) = self.nodes.remove(&current) else {
                continue;
            };
            if node.has_list {
                renderer.release_boundary(BoundaryId(current.0));
            }
            if let Some(parent) = node.parent.and_then(|parent| self.nodes.get_mut(&parent)) {
                parent.children.retain(|child| *child != current);
            }
            stack.extend(node.children);
        }
    }

    /// Advances animations to `now`, re-records changed boundaries and composes the frame.
    pub fn prepare<R: Renderer>(
        &mut self,
        now: Duration,
        renderer: &mut R,
    ) -> (Composition, RenderStats) {
        self.frame += 1;
        let finished: Vec<NodeKey> = self
            .nodes
            .iter()
            .filter(|(_, node)| node.exiting && !node.values.is_animating())
            .map(|(key, _)| *key)
            .collect();
        for key in finished {
            self.remove_subtree(key, renderer);
        }
        self.flush_releases(renderer);
        let mut stats = RenderStats {
            nodes: self.nodes.len(),
            ..RenderStats::default()
        };
        let mut list = DrawList::new();
        for (key, node) in &mut self.nodes {
            let moving_frame = node.frame.tick(now);
            let moving_values = node.values.tick(now);
            stats.animating |= moving_frame || moving_values;
            let values = node.values.current();
            let size = node.frame.current().size;
            let paint_changed = node
                .recorded_values
                .is_none_or(|recorded| !recorded.paint_equals(&values));
            let size_changed = node.recorded_size != Some(size);
            if node.needs_record || paint_changed || size_changed {
                list.clear();
                record::record_node(node, &values, size, &mut list);
                if list.is_empty() {
                    if node.has_list {
                        renderer.release_boundary(BoundaryId(key.0));
                        node.has_list = false;
                    }
                } else {
                    renderer.record_boundary(BoundaryId(key.0), &list);
                    node.has_list = true;
                    stats.recorded += 1;
                }
                node.needs_record = false;
                node.recorded_values = Some(values);
                node.recorded_size = Some(size);
            }
        }
        let composition = compose::compose(self, self.clear_color);
        (composition, stats)
    }

    /// Whether any value is still animating.
    pub fn is_animating(&self) -> bool {
        self.nodes
            .values()
            .any(|node| node.frame.is_animating() || node.values.is_animating())
    }

    /// Number of nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the tree is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Frames prepared so far.
    pub fn frame_count(&self) -> u64 {
        self.frame
    }

    /// Introspection for devtools.
    pub fn inspect(&self) -> inspect::RenderTreeSnapshot {
        inspect::RenderTreeSnapshot {
            boundaries: self
                .nodes
                .iter()
                .filter(|(_, node)| node.has_list)
                .map(|(key, _)| inspect::BoundaryInfo {
                    id: key.0,
                    path: String::new(),
                    commands: 0,
                    recorded_at_frame: self.frame,
                })
                .collect(),
            active_animations: self
                .nodes
                .iter()
                .filter(|(_, node)| node.frame.is_animating() || node.values.is_animating())
                .map(|(key, _)| format!("#{}", key.0))
                .collect(),
        }
    }
}
