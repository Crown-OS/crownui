use geometry::Point;
use graphics::TextLayoutId;
use slotmap::SlotMap;

use crate::{
    PointerEvents,
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
}

impl ElementTree {
    pub(crate) fn insert(&mut self, node: ElementNode) -> NodeId {
        self.nodes.insert(node)
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

    pub(crate) fn iter_all_mut(&mut self) -> impl Iterator<Item = (NodeId, &mut ElementNode)> + '_ {
        self.nodes.iter_mut()
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
    }

    /// Removes `id` and its descendants; returns how many nodes were removed.
    pub(crate) fn remove_subtree(&mut self, id: NodeId) -> usize {
        if let Some(parent) = self.nodes.get(id).and_then(|node| node.parent)
            && let Some(parent) = self.nodes.get_mut(parent)
        {
            parent.children.retain(|child| *child != id);
            parent.dirty.children = true;
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
        let Some(root) = self.root else {
            return Vec::new();
        };
        let mut path = Vec::new();
        let hit_layer = self.top_layers(root).into_iter().find(|&layer| {
            self.hit_test_node(
                layer,
                point,
                self.inherited_pointer_events(layer),
                &mut path,
            )
        });
        if hit_layer.is_none() {
            self.hit_test_node(root, point, PointerEvents::Auto, &mut path);
        }
        path.reverse();
        path.extend(
            hit_layer
                .into_iter()
                .flat_map(|layer| self.ancestors(layer).skip(1)),
        );
        path
    }

    fn top_layers(&self, root: NodeId) -> Vec<NodeId> {
        let mut layers = Vec::new();
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            let Some(node) = self.nodes.get(id) else {
                continue;
            };
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
    ) -> bool {
        let Some(node) = self.nodes.get(id) else {
            return false;
        };
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
            if self.hit_test_node(*child, point, events, path) {
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
