use geometry::{CornerRadii, Rect, Size, Vector};
use graphics::{DrawCommand, DrawList};

use super::{VisualValues, tree::RenderNode};
use view::RenderContent;

fn grow_radii(radii: CornerRadii, amount: f32) -> CornerRadii {
    CornerRadii {
        top_left: (radii.top_left + amount).max(0.0),
        top_right: (radii.top_right + amount).max(0.0),
        bottom_right: (radii.bottom_right + amount).max(0.0),
        bottom_left: (radii.bottom_left + amount).max(0.0),
    }
}

/// Records a node's own box and content in node-local coordinates.
pub(crate) fn record_node(node: &RenderNode, values: &VisualValues, size: Size, list: &mut DrawList) {
    if node.visual.hidden {
        return;
    }
    let bounds = Rect::new(geometry::Point::ZERO, size);
    if values.shadow_color.a > 0.0 && (values.shadow_blur > 0.0 || values.shadow_offset != Vector::ZERO) {
        list.shadow(bounds, values.radii, values.shadow_color, values.shadow_blur, values.shadow_offset);
    }
    if values.background.a > 0.0 {
        list.fill(bounds, values.radii, values.background);
    }
    if values.border_width > 0.0 && values.border_color.a > 0.0 {
        let half = values.border_width / 2.0;
        let inset = bounds.inset(geometry::Insets::uniform(half));
        list.stroke(inset, grow_radii(values.radii, -half), values.border_width, values.border_color);
    }
    if values.ring_width > 0.0 && values.ring_color.a > 0.0 {
        let half = values.ring_width / 2.0;
        let outset = Rect::from_xywh(-half, -half, size.width + values.ring_width, size.height + values.ring_width);
        list.stroke(outset, grow_radii(values.radii, half), values.ring_width, values.ring_color);
    }
    match &node.content {
        RenderContent::None => {}
        RenderContent::Text { layout, origin } => list.text(*layout, *origin),
        RenderContent::Canvas(canvas) => list.extend_from(canvas),
        RenderContent::Image(image) => list.push(DrawCommand::Image(*image, bounds)),
    }
}
