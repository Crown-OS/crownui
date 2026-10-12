use geometry::{Point, Rect};
use graphics::{DrawList, ImageId, TextLayoutId};
use motion::{Curve, Transaction};
use slotmap::Key;
use windowing::{CursorIcon, MaterialShape, SurfaceState};

use crate::{
    NodeId,
    node::NodeContent,
    style::Visual,
    tree::ElementTree,
};

/// A render-thread node identity derived from the element's id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeKey(pub u64);

impl NodeKey {
    pub(crate) fn of(node: NodeId) -> Self {
        Self(node.data().as_ffi())
    }
}

/// What a render node draws inside its box.
#[derive(Debug, Clone, PartialEq)]
pub enum RenderContent {
    /// Nothing.
    None,
    /// Shaped text with its top-left corner at `origin` (node-relative).
    Text {
        /// Layout to draw.
        layout: TextLayoutId,
        /// Node-relative origin.
        origin: Point,
    },
    /// Custom drawing in node-relative coordinates.
    Canvas(DrawList),
    /// An image filling the node.
    Image(ImageId),
}

/// One change for the render thread.
#[derive(Debug, Clone, PartialEq)]
pub enum RenderOp {
    /// The tree's root.
    SetRoot(NodeKey),
    /// A node's children, in paint order before z-index.
    SetChildren(NodeKey, Vec<NodeKey>),
    /// A node and its descendants went away; with an exit visual, they stay drawn until it is reached.
    Remove(NodeKey, Option<(Visual, Transaction)>),
    /// A node's parent-relative frame, animated when a transaction is given.
    Frame(NodeKey, Rect, Option<Transaction>),
    /// A node's visual, animated when a transaction is given.
    Visual(NodeKey, Visual, Option<Transaction>),
    /// A node's content.
    Content(NodeKey, RenderContent),
}

/// Everything the UI thread hands over for one frame.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UiFrame {
    /// Tree changes in order.
    pub ops: Vec<RenderOp>,
    /// Surface state to commit with the frame (materials).
    pub surface: Option<SurfaceState>,
    /// New cursor, when it changed.
    pub cursor: Option<CursorIcon>,
    /// Images to upload before drawing.
    pub images: Vec<crate::cx::ImageUpload>,
    /// Images to free once no node draws them.
    pub image_releases: Vec<ImageId>,
}

impl UiFrame {
    /// Whether nothing changed.
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
            && self.surface.is_none()
            && self.cursor.is_none()
            && self.images.is_empty()
            && self.image_releases.is_empty()
    }
}

impl ElementTree {
    /// Emits ops for every change since the last call.
    pub(crate) fn collect_render_ops(&mut self, explicit: Option<Transaction>, emulate_materials: bool, ops: &mut Vec<RenderOp>) {
        let Some(root) = self.root() else {
            return;
        };
        let mut stack: Vec<(NodeId, Option<Curve>)> = vec![(root, None)];
        let mut root_created = false;
        while let Some((id, inherited)) = stack.pop() {
            let Some(element) = self.get_mut(id) else {
                continue;
            };
            let key = NodeKey::of(id);
            let curve = element.transition.or(inherited);
            let transaction = explicit.or(curve.map(Transaction::new));
            let newly_created = !element.sent.created;
            if newly_created {
                element.sent.created = true;
                root_created |= id == root;
                if let Some(enter) = element.enter.take() {
                    let mut start = element.style.clone();
                    for prop in &enter {
                        prop.apply(&mut start);
                    }
                    ops.push(RenderOp::Visual(key, start.visual, None));
                    let entering = transaction.or(Some(Transaction::new(motion::Spring::smooth())));
                    ops.push(RenderOp::Visual(key, element.visual, entering));
                    element.sent.visual = Some(element.visual);
                    element.dirty.visual = false;
                }
            }
            if element.dirty.frame || newly_created {
                let rect = element.geometry.rect;
                if element.sent.frame != Some(rect) {
                    let animate = if newly_created { None } else { transaction };
                    ops.push(RenderOp::Frame(key, rect, animate));
                    element.sent.frame = Some(rect);
                }
                element.dirty.frame = false;
            }
            if element.dirty.visual || newly_created {
                let visual = match element.style.material.filter(|_| emulate_materials) {
                    Some(material) => emulated_material(element.visual, &material),
                    None => element.visual,
                };
                if element.sent.visual != Some(visual) {
                    let animate = if newly_created { None } else { transaction };
                    ops.push(RenderOp::Visual(key, visual, animate));
                    element.sent.visual = Some(visual);
                }
                element.dirty.visual = false;
            }
            if element.dirty.content || newly_created {
                let content = match &element.content {
                    NodeContent::Container => RenderContent::None,
                    NodeContent::Text(text) => match text.layout {
                        Some(layout) => RenderContent::Text {
                            layout,
                            origin: Point::new(element.style.layout.padding.left, element.style.layout.padding.top),
                        },
                        None => RenderContent::None,
                    },
                    NodeContent::Canvas(canvas) => RenderContent::Canvas(canvas.list.clone()),
                    NodeContent::Image(image) => RenderContent::Image(image.image),
                };
                ops.push(RenderOp::Content(key, content));
                element.dirty.content = false;
            }
            let children_changed = element.dirty.children || newly_created;
            element.dirty.children = false;
            let children = element.children.clone();
            let child_curve = element.transition.or(inherited);
            if children_changed {
                ops.push(RenderOp::SetChildren(key, children.iter().map(|child| NodeKey::of(*child)).collect()));
            }
            stack.extend(children.into_iter().rev().map(|child| (child, child_curve)));
        }
        if root_created {
            ops.push(RenderOp::SetRoot(NodeKey::of(root)));
        }
    }

    /// Window-space shapes of every node with a platform material, and the first material.
    pub(crate) fn material_shapes(&self) -> (Option<windowing::Material>, Vec<MaterialShape>) {
        let mut material = None;
        let shapes = self
            .iter()
            .filter_map(|(_, element)| {
                let node_material = element.style.material?;
                material.get_or_insert(node_material);
                (!element.visual.hidden).then_some(MaterialShape {
                    rect: element.geometry.window_rect,
                    radii: element.visual.radii,
                })
            })
            .collect();
        (material, shapes)
    }
}


/// Approximates a compositor material in-app: backdrop blur plus the material's tint.
fn emulated_material(mut visual: Visual, material: &windowing::Material) -> Visual {
    if let Some(blur) = material.blur {
        visual.backdrop_blur = visual.backdrop_blur.max(blur.radius);
        visual.background = visual.background.or(Some(blur.tint));
    }
    if visual.radii.is_zero() {
        visual.radii = geometry::CornerRadii::uniform(material.corner_radius);
    }
    visual
}
