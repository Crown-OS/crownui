use std::collections::HashMap;

use geometry::{CornerRadii, Transform};
use graphics::{CompositeOp, LayerProps};
use inspect::LayoutSnapshot;
use semantics::SemanticTree;

use crate::{
    draw_items::{DrawSources, draw_items, same_box},
    renderer::CaptureRenderer,
    scene::{Item, SceneNode},
};

struct NodeInfo<'a> {
    path: &'a str,
    rect: [f32; 4],
}

enum Open {
    Node(SceneNode),
    Group(LayerProps, Vec<Item>),
}

/// Rebuilds the view tree from the last composition: one [`SceneNode`] per view layer, clip and
/// scroll layers as groups, each boundary's commands as the node's items.
pub(crate) fn scene_tree(
    renderer: &CaptureRenderer,
    image_keys: &HashMap<graphics::ImageId, String>,
    layout: &LayoutSnapshot,
    semantics: &SemanticTree,
    width: f32,
    height: f32,
) -> SceneNode {
    let nodes: HashMap<u64, NodeInfo<'_>> = layout
        .nodes
        .iter()
        .map(|node| {
            (
                node.key,
                NodeInfo {
                    path: &node.path,
                    rect: node.rect,
                },
            )
        })
        .collect();
    let labels: HashMap<&str, String> = semantics
        .iter()
        .filter_map(|node| {
            let name = node
                .name
                .as_deref()
                .filter(|name| !name.trim().is_empty())?;
            Some((
                node.path.as_str(),
                format!("{} \"{}\"", node.role, shorten(name)),
            ))
        })
        .collect();
    let sources = DrawSources {
        texts: &renderer.texts,
        image_keys,
    };
    let mut stack = vec![Open::Node(SceneNode {
        name: "window".to_owned(),
        path: String::new(),
        matrix: Transform::IDENTITY.matrix,
        width,
        height,
        opacity: 1.0,
        clip: true,
        ..SceneNode::default()
    })];
    for op in &renderer.composition.ops {
        match op {
            CompositeOp::PushLayer(props) => stack.push(match props.node {
                Some(key) => Open::Node(open_node(props, nodes.get(&key.0), &labels)),
                None => Open::Group(*props, Vec::new()),
            }),
            CompositeOp::Draw(id) => {
                if let (Some(list), Some(open)) = (renderer.boundaries.get(id), stack.last_mut()) {
                    items_of(open).extend(draw_items(list, &sources));
                }
            }
            CompositeOp::PopLayer => {
                if stack.len() > 1
                    && let Some(open) = stack.pop()
                    && let Some(item) = close(open)
                {
                    let item = if stack.len() == 1 {
                        as_top_layer(item)
                    } else {
                        item
                    };
                    if let Some(parent) = stack.last_mut() {
                        items_of(parent).push(item);
                    }
                }
            }
        }
    }
    while stack.len() > 1 {
        if let Some(item) = stack.pop().and_then(close)
            && let Some(parent) = stack.last_mut()
        {
            items_of(parent).push(item);
        }
    }
    match stack.pop() {
        Some(Open::Node(window)) => window,
        _ => SceneNode::default(),
    }
}

fn shorten(name: &str) -> String {
    const LIMIT: usize = 48;
    let line = name.lines().next().unwrap_or_default();
    if line.chars().count() > LIMIT {
        format!("{}…", line.chars().take(LIMIT).collect::<String>())
    } else {
        line.to_owned()
    }
}

fn open_node(
    props: &LayerProps,
    info: Option<&NodeInfo<'_>>,
    labels: &HashMap<&str, String>,
) -> SceneNode {
    let path = info.map(|info| info.path).unwrap_or_default();
    let element = path.rsplit(" > ").next().unwrap_or("view");
    let [.., width, height] = info.map_or([0.0; 4], |info| info.rect);
    let (clip, radii) = props
        .clip
        .map_or((false, [0.0; 4]), |(_, radii)| (true, radii_array(radii)));
    SceneNode {
        name: labels
            .get(path)
            .cloned()
            .unwrap_or_else(|| element.to_owned()),
        path: path.to_owned(),
        matrix: props.transform.matrix,
        width,
        height,
        opacity: props.opacity,
        radii,
        clip,
        backdrop_blur: props.backdrop_blur,
        ..SceneNode::default()
    }
}

fn items_of(open: &mut Open) -> &mut Vec<Item> {
    match open {
        Open::Node(node) => &mut node.items,
        Open::Group(_, items) => items,
    }
}

fn close(open: Open) -> Option<Item> {
    match open {
        Open::Node(mut node) => {
            hoist_box(&mut node);
            absorb_child_clip(&mut node);
            Some(Item::Node(node))
        }
        Open::Group(_, items) if items.is_empty() => None,
        Open::Group(props, items) => Some(Item::Group {
            name: group_name(&props).to_owned(),
            matrix: props.transform.matrix,
            clip: props.clip.map(|(rect, _)| {
                [
                    rect.origin.x,
                    rect.origin.y,
                    rect.size.width,
                    rect.size.height,
                ]
            }),
            radii: props.clip.map_or([0.0; 4], |(_, radii)| radii_array(radii)),
            opacity: props.opacity,
            backdrop_blur: props.backdrop_blur,
            items,
        }),
    }
}

/// Popovers, menus and toasts paint after the tree from window-level layers.
fn as_top_layer(item: Item) -> Item {
    match item {
        Item::Group {
            name,
            matrix,
            clip,
            radii,
            opacity,
            backdrop_blur,
            items,
        } if name == "offset" => Item::Group {
            name: "top layer".to_owned(),
            matrix,
            clip,
            radii,
            opacity,
            backdrop_blur,
            items,
        },
        item => item,
    }
}

fn group_name(props: &LayerProps) -> &'static str {
    if props.clip.is_some() {
        "clip"
    } else if props.transform.is_identity() {
        "layer"
    } else {
        "offset"
    }
}

fn radii_array(radii: CornerRadii) -> [f32; 4] {
    [
        radii.top_left,
        radii.top_right,
        radii.bottom_right,
        radii.bottom_left,
    ]
}

/// Moves a leading rectangle covering the whole node into the node's own fill, outline and
/// shadows, as a designer would draw the box.
fn hoist_box(node: &mut SceneNode) {
    let Some(Item::Rect { rect, .. }) = node.items.first() else {
        return;
    };
    let rect = *rect;
    if node.width <= 0.0 && rect[0].abs() < 0.5 && rect[1].abs() < 0.5 {
        node.width = rect[2];
        node.height = rect[3];
    }
    if !same_box(rect, [0.0, 0.0, node.width, node.height]) {
        return;
    }
    if let Item::Rect {
        radii,
        fill,
        stroke,
        shadows,
        ..
    } = node.items.remove(0)
    {
        if radii != [0.0; 4] || !node.clip {
            node.radii = radii;
        }
        node.fill = fill;
        node.stroke = stroke;
        node.shadows = shadows;
    }
}

/// Folds an `overflow: hidden` clip layer spanning the whole node, the last thing it draws, into
/// the node itself.
fn absorb_child_clip(node: &mut SceneNode) {
    let is_own_clip = matches!(
        node.items.last(),
        Some(Item::Group { name, clip: Some(clip), matrix, opacity, backdrop_blur: None, .. })
            if name == "clip"
                && *opacity == 1.0
                && *matrix == Transform::IDENTITY.matrix
                && same_box(*clip, [0.0, 0.0, node.width, node.height])
    );
    if !is_own_clip {
        return;
    }
    if let Some(Item::Group { radii, items, .. }) = node.items.pop() {
        if radii != [0.0; 4] {
            node.radii = radii;
        }
        node.clip = true;
        node.items.extend(items);
    }
}
