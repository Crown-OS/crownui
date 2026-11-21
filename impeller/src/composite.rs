use geometry::CornerShape;
use graphics::{BoundaryId, CompositeOp, Composition, LayerProps};
use impellers::{
    BlendMode, ClipOperation, DisplayList, DisplayListBuilder, ImageFilter, TextureSampling,
};
use rustc_hash::FxHashMap;

use crate::{
    translate::{opacity_paint, save_layer, unbounded_rect},
    util::{
        geometry::{to_color, to_matrix, to_projective_matrix, to_rect},
        outline::Outline,
        sdf_shape::SdfShape,
        shape_shader::ShapeShader,
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
/// boundaries are skipped. Layers are cut to rounded clips by `shader` when there is one, for
/// antialiased corners; without it rounded clips use the stencil (MSAA).
pub(crate) fn compose_frame(
    composition: &Composition,
    boundaries: &FxHashMap<BoundaryId, DisplayList>,
    viewport: Viewport,
    corners: CornerShape,
    shader: Option<ShapeShader<'_>>,
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
    let mut open_layers = Vec::new();
    for op in &composition.ops {
        match op {
            CompositeOp::PushLayer(props) => {
                let save_count = builder.get_save_count();
                let mask = push_layer(&mut builder, props, corners, shader);
                open_layers.push((save_count, mask));
            }
            CompositeOp::PopLayer => {
                if let Some((save_count, mask)) = open_layers.pop() {
                    if let Some((mask, shader)) = mask.zip(shader) {
                        builder.restore_to_count(mask.depth);
                        mask.shape.mask(&mut builder, 0.0, shader);
                    }
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

/// A rounded clip cut out of a layer when it closes, once the layers above `depth` are gone.
struct LayerMask {
    shape: SdfShape,
    depth: u32,
}

/// Opens a layer; returns the shape to cut it to when it closes. A rounded clip on a layer that
/// would not otherwise go offscreen gets a layer bounded to it, so the stencil never cuts it.
fn push_layer(
    builder: &mut DisplayListBuilder,
    props: &LayerProps,
    corners: CornerShape,
    shader: Option<ShapeShader<'_>>,
) -> Option<LayerMask> {
    builder.save();
    if let Some(tilt) = props.tilt {
        builder.transform(&to_projective_matrix(tilt));
    }
    let image = props.transform_as_image && !props.transform.is_translation();
    let opacity = if image {
        save_image_layer(builder, props);
        1.0
    } else {
        if !props.transform.is_identity() {
            builder.transform(&to_matrix(props.transform));
        }
        props.opacity
    };
    let shape = match props.clip {
        Some((rect, radii)) if shader.is_some() && !radii.is_zero() => {
            builder.clip_rect(&to_rect(rect), ClipOperation::Intersect);
            Some(SdfShape::new(rect, radii, corners))
        }
        Some((rect, radii)) => {
            Outline::new(rect, radii, corners).clip(builder);
            None
        }
        None => None,
    };
    let offscreen = save_layer(
        builder,
        props.clip.map(|(rect, _)| rect),
        opacity,
        props.backdrop_blur,
    );
    if let Some((rect, _)) = props.clip.filter(|_| shape.is_some() && !offscreen) {
        builder.save_layer(&to_rect(rect), None, None);
    }
    let depth = builder.get_save_count();
    shape.map(|shape| LayerMask { shape, depth })
}

/// Opens the layer `props.transform` moves as a drawn image, faded by `props.opacity`.
fn save_image_layer(builder: &mut DisplayListBuilder, props: &LayerProps) {
    let mut paint = opacity_paint(props.opacity);
    paint.set_image_filter(&ImageFilter::new_matrix(
        &to_matrix(props.transform),
        TextureSampling::Linear,
    ));
    let bounds = props
        .clip
        .map_or_else(unbounded_rect, |(rect, _)| to_rect(rect));
    builder.save_layer(&bounds, Some(&paint), None);
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
            &mut DrawResources::default(),
            None,
            CornerShape::Squircle,
            2.0,
        ) else {
            panic!("a fill records");
        };
        let boundaries = FxHashMap::from_iter([(BoundaryId(1), recorded)]);
        let layer = LayerProps {
            transform: Transform::translate(Vector::new(10.0, 20.0)),
            transform_as_image: false,
            tilt: None,
            opacity: 0.8,
            clip: Some((
                Rect::from_xywh(0.0, 0.0, 100.0, 100.0),
                CornerRadii::uniform(12.0),
            )),
            backdrop_blur: Some(16.0),
            node: None,
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
            assert!(compose_frame(&composition, &boundaries, viewport, corners, None).is_some());
        }
    }
}
