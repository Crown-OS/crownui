use geometry::{Point, Vector};
use graphics::TextLayoutId;
use input::{Key, Keybind, KeyboardEvent};
use rustc_hash::FxHashMap;
use slotmap::SlotMap;
use smallvec::SmallVec;

use crate::{
    PointerEvents,
    dirty::DirtyQueues,
    node::{ElementNode, NodeContent, NodeId},
};

/// A node that left the tree since the last frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RemovedNode {
    pub(crate) id: NodeId,
    pub(crate) rendered: bool,
    pub(crate) text_layout: Option<TextLayoutId>,
    pub(crate) exit: Option<(crate::style::Visual, Option<motion::Curve>)>,
    pub(crate) inside_exit: bool,
}

/// Every mounted element, owned by the context.
#[derive(Default)]
pub struct ElementTree {
    nodes: SlotMap<NodeId, ElementNode>,
    root: Option<NodeId>,
    removed: Vec<RemovedNode>,
    pub(crate) queues: DirtyQueues,
    /// Nodes styled with a platform material, so their shapes are found without a walk.
    pub(crate) material_nodes: Vec<NodeId>,
    /// Nodes by their shortcut, letters lowercased, so a key press finds its target directly.
    shortcuts: FxHashMap<Keybind, SmallVec<[NodeId; 1]>>,
    /// Nodes whose window rect moved outside a layout pass (scrolling), for semantics to follow.
    pub(crate) moved_outside_layout: Vec<NodeId>,
    /// Nodes visited while translating scrolled subtrees since the counters were last taken.
    pub(crate) scroll_visits: u32,
}

/// Letters compare case-insensitively (see [`Keybind::matches`]), so shortcuts index lowercase.
fn shortcut_key(keybind: Keybind) -> Keybind {
    match keybind.key {
        Key::Character(letter) => Keybind {
            key: Key::Character(letter.to_ascii_lowercase()),
            modifiers: keybind.modifiers,
        },
        _ => keybind,
    }
}

impl ElementTree {
    pub(crate) fn insert(&mut self, node: ElementNode) -> NodeId {
        let id = self.nodes.insert(node);
        self.mark_layout(id);
        self.mark_render(id);
        id
    }

    /// Queues `id` for the next layout pass and tells its ancestors the write-back must descend.
    pub(crate) fn mark_layout(&mut self, id: NodeId) {
        let Some(node) = self.nodes.get_mut(id) else {
            return;
        };
        if !node.queued.layout {
            node.queued.layout = true;
            self.queues.layout.push(id);
        }
        let mut current = Some(id);
        while let Some(id) = current {
            let Some(node) = self.nodes.get_mut(id) else {
                break;
            };
            if node.queued.needs_visit {
                break;
            }
            node.queued.needs_visit = true;
            current = node.parent;
        }
    }

    /// Queues `id` for the next render-op collection.
    pub(crate) fn mark_render(&mut self, id: NodeId) {
        if let Some(node) = self.nodes.get_mut(id)
            && !node.queued.render
        {
            node.queued.render = true;
            self.queues.render.push(id);
        }
    }

    /// Shifts every descendant of `node` by `delta` in window space, as scrolling does: their
    /// parent-relative rects and the layout engine are untouched, so no layout pass follows.
    pub(crate) fn translate_descendants(&mut self, node: NodeId, delta: Vector) {
        let Some(parent) = self.nodes.get(node) else {
            return;
        };
        let mut stack: Vec<NodeId> = parent.children.clone();
        while let Some(id) = stack.pop() {
            let Some(element) = self.nodes.get_mut(id) else {
                continue;
            };
            self.scroll_visits += 1;
            element.geometry.window_rect = element.geometry.window_rect.translate(delta);
            stack.extend(element.children.iter().copied());
            self.moved_outside_layout.push(id);
        }
    }

    /// Whether a layout pass has anything to do.
    pub(crate) fn has_layout_work(&self) -> bool {
        !self.queues.layout.is_empty() || !self.removed.is_empty()
    }

    /// Whether any node waits for layout or for the renderer.
    pub(crate) fn has_dirty(&self) -> bool {
        self.has_layout_work() || !self.queues.render.is_empty()
    }

    /// The nearest transition set on an ancestor of `id`.
    pub(crate) fn inherited_transition(&self, id: NodeId) -> Option<motion::Curve> {
        self.ancestors(id)
            .skip(1)
            .find_map(|ancestor| self.nodes.get(ancestor)?.transition)
    }

    /// Records that `node` activates on `keybind`.
    pub(crate) fn index_shortcut(&mut self, node: NodeId, keybind: Keybind) {
        let targets = self.shortcuts.entry(shortcut_key(keybind)).or_default();
        if !targets.contains(&node) {
            targets.push(node);
        }
    }

    /// The first enabled node whose shortcut `event` presses.
    pub(crate) fn shortcut_target(&self, event: &KeyboardEvent) -> Option<NodeId> {
        let pressed = shortcut_key(Keybind {
            key: event.key,
            modifiers: event.modifiers,
        });
        self.shortcuts.get(&pressed)?.iter().copied().find(|node| {
            self.nodes.get(*node).is_some_and(|element| {
                !element.disabled
                    && element
                        .shortcut
                        .is_some_and(|keybind| keybind.matches(event))
            })
        })
    }

    /// Lists `id` among the material nodes, or drops it.
    pub(crate) fn track_material(&mut self, id: NodeId, has_material: bool) {
        let listed = self.material_nodes.iter().position(|node| *node == id);
        match (has_material, listed) {
            (true, None) => self.material_nodes.push(id),
            (false, Some(index)) => {
                self.material_nodes.swap_remove(index);
            }
            _ => {}
        }
    }

    /// The root element.
    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    pub(crate) fn set_root(&mut self, root: NodeId) {
        self.root = Some(root);
    }

    /// Looks up a node.
    pub fn get(&self, id: NodeId) -> Option<&ElementNode> {
        self.nodes.get(id)
    }

    pub(crate) fn get_mut(&mut self, id: NodeId) -> Option<&mut ElementNode> {
        self.nodes.get_mut(id)
    }

    /// Number of mounted nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether nothing is mounted.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Every node, in no particular order.
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &ElementNode)> + '_ {
        self.nodes.iter()
    }

    pub(crate) fn set_children(&mut self, parent: NodeId, children: Vec<NodeId>) {
        for child in &children {
            if let Some(node) = self.nodes.get_mut(*child) {
                node.parent = Some(parent);
            }
        }
        if let Some(node) = self.nodes.get_mut(parent) {
            node.children = children;
            node.dirty.children = true;
        }
        self.mark_layout(parent);
        self.mark_render(parent);
    }

    /// Removes `id` and its descendants; returns how many nodes were removed.
    pub(crate) fn remove_subtree(&mut self, id: NodeId) -> usize {
        let parent = self.nodes.get(id).and_then(|node| node.parent);
        if let Some(parent) = parent.and_then(|parent| self.nodes.get_mut(parent)) {
            parent.children.retain(|child| *child != id);
            parent.dirty.children = true;
        }
        if let Some(parent) = parent {
            self.mark_layout(parent);
            self.mark_render(parent);
        }
        let inherited_curve = self.ancestors(id).find_map(|ancestor| {
            self.nodes
                .get(ancestor)
                .and_then(|element| element.transition)
        });
        let root_exits = self
            .nodes
            .get(id)
            .is_some_and(|element| element.exit.is_some());
        let mut stack = vec![id];
        let mut count = 0;
        while let Some(current) = stack.pop() {
            if let Some(node) = self.nodes.remove(current) {
                let exit = (current == id)
                    .then_some(node.exit.as_ref())
                    .flatten()
                    .map(|props| {
                        let mut style = node.style.clone();
                        style.visual = node.visual;
                        for prop in props {
                            prop.apply(&mut style);
                        }
                        (style.visual, inherited_curve)
                    });
                let text_layout = match &node.content {
                    NodeContent::Text(text) => text.layout,
                    _ => None,
                };
                self.removed.push(RemovedNode {
                    id: current,
                    rendered: node.sent.created,
                    text_layout,
                    exit,
                    inside_exit: root_exits && current != id,
                });
                stack.extend(node.children);
                count += 1;
                if !self.material_nodes.is_empty() {
                    self.material_nodes.retain(|node| *node != current);
                }
                if let Some(keybind) = node.shortcut
                    && let Some(targets) = self.shortcuts.get_mut(&shortcut_key(keybind))
                {
                    targets.retain(|target| *target != current);
                }
            }
        }
        count
    }

    pub(crate) fn take_removed(&mut self) -> Vec<RemovedNode> {
        std::mem::take(&mut self.removed)
    }

    /// Ancestors of `id`, nearest first, including `id`.
    pub fn ancestors(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        let mut current = Some(id);
        std::iter::from_fn(move || {
            let node = current?;
            current = self.nodes.get(node).and_then(|element| element.parent);
            Some(node)
        })
    }

    /// The component path of `id`, e.g. `vstack > hstack[1] > button`.
    pub fn path(&self, id: NodeId) -> String {
        let mut segments: Vec<String> = self
            .ancestors(id)
            .filter_map(|node| {
                let element = self.nodes.get(node)?;
                let index = element
                    .parent
                    .and_then(|parent| self.nodes.get(parent))
                    .and_then(|parent| parent.children.iter().position(|child| *child == node));
                Some(match (element.key, index) {
                    (Some(key), _) => format!("{}[#{key}]", element.name),
                    (None, Some(index)) if index > 0 => format!("{}[{index}]", element.name),
                    _ => element.name.to_owned(),
                })
            })
            .collect();
        segments.reverse();
        segments.join(" > ")
    }

    /// Window-space boxes of the visible nodes that opt back into pointer events under a root with
    /// `pointer-events: none`, or `None` when the root takes pointer events everywhere.
    pub fn input_region(&self) -> Option<Vec<geometry::Rect>> {
        let root = self.root?;
        if self.nodes.get(root)?.style.pointer_events != Some(PointerEvents::None) {
            return None;
        }
        let mut region = Vec::new();
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            let Some(node) = self.nodes.get(id) else {
                continue;
            };
            if node.visual.hidden {
                continue;
            }
            if node.style.pointer_events == Some(PointerEvents::Auto) {
                region.push(node.geometry.window_rect);
            }
            stack.extend(node.children.iter().copied());
        }
        Some(region)
    }

    /// The deepest visible node under `point` that takes pointer events, followed by its ancestors.
    pub fn hit_test(&self, point: Point) -> Vec<NodeId> {
        self.hit_test_counted(point, &mut 0)
    }

    /// [`ElementTree::hit_test`], adding the nodes it looked at to `visits`.
    pub(crate) fn hit_test_counted(&self, point: Point, visits: &mut u32) -> Vec<NodeId> {
        let Some(root) = self.root else {
            return Vec::new();
        };
        let mut path = Vec::new();
        let hit_layer = self.top_layers(root, visits).into_iter().find(|&layer| {
            self.hit_test_node(
                layer,
                point,
                self.inherited_pointer_events(layer),
                &mut path,
                visits,
            )
        });
        if hit_layer.is_none() {
            self.hit_test_node(root, point, PointerEvents::Auto, &mut path, visits);
        }
        path.reverse();
        path.extend(
            hit_layer
                .into_iter()
                .flat_map(|layer| self.ancestors(layer).skip(1)),
        );
        path
    }

    fn top_layers(&self, root: NodeId, visits: &mut u32) -> Vec<NodeId> {
        let mut layers = Vec::new();
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            let Some(node) = self.nodes.get(id) else {
                continue;
            };
            *visits += 1;
            if node.visual.hidden {
                continue;
            }
            if node.visual.top_layer {
                layers.push(id);
            }
            stack.extend(node.children.iter().copied());
        }
        layers
    }

    fn inherited_pointer_events(&self, id: NodeId) -> PointerEvents {
        self.ancestors(id)
            .skip(1)
            .find_map(|ancestor| self.nodes.get(ancestor)?.style.pointer_events)
            .unwrap_or(PointerEvents::Auto)
    }

    fn hit_test_node(
        &self,
        id: NodeId,
        point: Point,
        inherited: PointerEvents,
        path: &mut Vec<NodeId>,
        visits: &mut u32,
    ) -> bool {
        let Some(node) = self.nodes.get(id) else {
            return false;
        };
        *visits += 1;
        if node.visual.hidden {
            return false;
        }
        let events = node.style.pointer_events.unwrap_or(inherited);
        let inside = events == PointerEvents::Auto && node.geometry.window_rect.contains(point);
        if node.visual.clip && !node.geometry.window_rect.contains(point) {
            return false;
        }
        let mut ordered: Vec<NodeId> = node
            .children
            .iter()
            .copied()
            .filter(|child| {
                self.nodes
                    .get(*child)
                    .is_some_and(|child| !child.visual.top_layer)
            })
            .collect();
        ordered.sort_by_key(|child| {
            self.nodes
                .get(*child)
                .map_or(0, |child| child.visual.z_index)
        });
        path.push(id);
        for child in ordered.iter().rev() {
            if self.hit_test_node(*child, point, events, path, visits) {
                return true;
            }
        }
        if inside {
            return true;
        }
        path.pop();
        false
    }
}
