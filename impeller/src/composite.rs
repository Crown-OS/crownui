use std::collections::HashMap;

use geometry::CornerShape;
use graphics::{BoundaryId, CompositeOp, Composition, LayerProps};
use impellers::{BlendMode, DisplayList, DisplayListBuilder};

use crate::{
    translate::save_layer,
    util::{
        geometry::{to_color, to_matrix},
        outline::Outline,
    },
};

/// Drawable size in device pixels and the logical-to-device scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Viewport {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) scale: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            scale: 1.0,
        }
    }
}

impl Viewport {
    pub(crate) fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }
}

/// Builds one frame: clears to the composition's color, scales logical pixels to device pixels
/// and replays the cached boundaries inside their layers, clipped with `corners`; missing
/// boundaries are skipped.
pub(crate) fn compose_frame(
    composition: &Composition,
    boundaries: &HashMap<BoundaryId, DisplayList>,
    viewport: Viewport,
    corners: CornerShape,
) -> Option<DisplayList> {
    let device_bounds = impellers::Rect::new(
        impellers::Point::zero(),
        impellers::Size::new(viewport.width as f32, viewport.height as f32),
    );
    let mut builder = DisplayListBuilder::new(Some(&device_bounds));
    let mut clear = impellers::Paint::default();
    clear
        .set_color(to_color(composition.clear_color))
        .set_blend_mode(BlendMode::Source);
    builder.draw_paint(&clear);
    builder.scale(viewport.scale, viewport.scale);
    let mut layer_save_counts = Vec::new();
    for op in &composition.ops {
        match op {
            CompositeOp::PushLayer(props) => {
                layer_save_counts.push(builder.get_save_count());
                push_layer(&mut builder, props, corners);
            }
            CompositeOp::PopLayer => {
                if let Some(save_count) = layer_save_counts.pop() {
                    builder.restore_to_count(save_count);
                }
            }
            CompositeOp::Draw(boundary) => {
                if let Some(list) = boundaries.get(boundary) {
                    builder.draw_display_list(list, 1.0);
                }
            }
        }
    }
    builder.build()
}

fn push_layer(builder: &mut DisplayListBuilder, props: &LayerProps, corners: CornerShape) {
    builder.save();
    if !props.transform.is_identity() {
        builder.transform(&to_matrix(props.transform));
    }
    if let Some((rect, radii)) = props.clip {
        Outline::new(rect, radii, corners).clip(builder);
    }
    save_layer(
        builder,
        props.clip.map(|(rect, _)| rect),
        props.opacity,
        props.backdrop_blur,
    );
}

#[cfg(test)]
mod tests {
    use geometry::{CornerRadii, Rect, Transform, Vector, rgba};
    use graphics::DrawList;

    use super::*;
    use crate::{resources::DrawResources, translate::translate_draw_list};

    #[test]
    fn compositions_with_nested_layers_build_a_frame() {
        let mut list = DrawList::new();
        list.fill(
            Rect::from_xywh(0.0, 0.0, 50.0, 50.0),
            CornerRadii::uniform(8.0),
            rgba(0x3b82f6ff),
        );
        let Some(recorded) = translate_draw_list(
            &list,
            &DrawResources::default(),
            None,
            CornerShape::Squircle,
        ) else {
            panic!("a fill records");
        };
        let boundaries = HashMap::from([(BoundaryId(1), recorded)]);
        let layer = LayerProps {
            transform: Transform::translate(Vector::new(10.0, 20.0)),
            opacity: 0.8,
            clip: Some((
                Rect::from_xywh(0.0, 0.0, 100.0, 100.0),
                CornerRadii::uniform(12.0),
            )),
            backdrop_blur: Some(16.0),
        };
        let composition = Composition {
            clear_color: rgba(0x00000000),
            ops: vec![
                CompositeOp::PushLayer(layer),
                CompositeOp::Draw(BoundaryId(1)),
                CompositeOp::PushLayer(LayerProps::default()),
                CompositeOp::Draw(BoundaryId(2)),
                CompositeOp::PopLayer,
                CompositeOp::PopLayer,
                CompositeOp::PopLayer,
                CompositeOp::PushLayer(LayerProps::default()),
            ],
        };
        let viewport = Viewport {
            width: 800,
            height: 600,
            scale: 2.0,
        };
        for corners in [CornerShape::Round, CornerShape::Squircle] {
            assert!(compose_frame(&composition, &boundaries, viewport, corners).is_some());
        }
    }
}
