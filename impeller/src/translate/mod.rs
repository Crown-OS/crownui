mod layer;
mod paint;

use geometry::{CornerRadii, CornerShape, Rect, WHITE};
use graphics::{Brush, DrawCommand, DrawList, Paint, Path, Stroke};
use impellers::{
    BlurStyle, ClipOperation, Context, DisplayList, DisplayListBuilder, MaskFilter, TextureSampling,
};

pub(crate) use layer::save_layer;
use paint::{fill_paint, stroke_paint};

use crate::{
    resources::DrawResources,
    util::{
        blur::shadow_sigma,
        geometry::{max_scale, to_color, to_matrix, to_path, to_point, to_rect},
        outline::Outline,
        path_shader::PathShader,
        sdf_path::SdfPath,
        sdf_shape::SdfShape,
    },
};

/// Records `list` as an Impeller display list, resolving text and images through `resources`
/// and drawing rounded corners as `corners`; commands naming missing resources are skipped and
/// unbalanced restores ignored. With a `context` current on this thread, rounded shapes and
/// paths small on screen at `scale` device pixels per logical pixel get analytic coverage from
/// the resources' shaders, uploading path segments into `resources`; without one they rely on
/// MSAA.
pub(crate) fn translate_draw_list(
    list: &DrawList,
    resources: &mut DrawResources,
    context: Option<&Context>,
    corners: CornerShape,
    scale: f32,
) -> Option<DisplayList> {
    let mut recorder = Recorder {
        builder: DisplayListBuilder::new(None),
        saved_scales: Vec::new(),
        device_scale: scale,
        resources,
        context,
        corners,
    };
    for command in list.commands() {
        recorder.record(command);
    }
    recorder.builder.build()
}

struct Recorder<'frame> {
    builder: DisplayListBuilder,
    /// The device scale at each open save, restored with it.
    saved_scales: Vec<f32>,
    /// Device pixels per unit under the current transform.
    device_scale: f32,
    resources: &'frame mut DrawResources,
    context: Option<&'frame Context>,
    corners: CornerShape,
}

impl Recorder<'_> {
    fn record(&mut self, command: &DrawCommand) {
        let builder = &mut self.builder;
        match command {
            DrawCommand::Save => {
                builder.save();
                self.saved_scales.push(self.device_scale);
            }
            DrawCommand::Restore => {
                if let Some(scale) = self.saved_scales.pop() {
                    builder.restore();
                    self.device_scale = scale;
                }
            }
            DrawCommand::SaveLayer {
                bounds,
                opacity,
                backdrop_blur,
            } => {
                if !save_layer(builder, *bounds, *opacity, *backdrop_blur) {
                    builder.save();
                }
                self.saved_scales.push(self.device_scale);
            }
            DrawCommand::Transform(transform) => {
                builder.transform(&to_matrix(*transform));
                self.device_scale *= max_scale(*transform);
            }
            DrawCommand::ClipRect(rect) => {
                builder.clip_rect(&to_rect(*rect), ClipOperation::Intersect);
            }
            DrawCommand::ClipRoundedRect(rect, radii) => {
                Outline::new(*rect, *radii, self.corners).clip(builder);
            }
            DrawCommand::FillRect(rect, paint) => {
                builder.draw_rect(&to_rect(*rect), &fill_paint(paint, self.resources));
            }
            DrawCommand::FillRoundedRect(rect, radii, paint) => {
                self.rounded_rect(*rect, *radii, paint, None);
            }
            DrawCommand::StrokeRoundedRect(rect, radii, stroke, paint) => {
                self.rounded_rect(*rect, *radii, paint, Some(*stroke));
            }
            DrawCommand::FillPath(path, paint) => {
                self.path(path, paint, None);
            }
            DrawCommand::StrokePath(path, stroke, paint) => {
                self.path(path, paint, Some(*stroke));
            }
            DrawCommand::Shadow {
                rect,
                radii,
                color,
                blur,
                offset,
            } => {
                let rect = rect.translate(*offset);
                let Some(sigma) = shadow_sigma(*blur) else {
                    self.rounded_rect(rect, *radii, &Paint::solid(*color), None);
                    return;
                };
                let mut paint = impellers::Paint::default();
                paint.set_color(to_color(*color));
                paint.set_mask_filter(&MaskFilter::new_blur(BlurStyle::Normal, sigma));
                Outline::new(rect, *radii, self.corners).draw(builder, &paint);
            }
            DrawCommand::Text(id, origin) => {
                if let Some(paragraph) = self.resources.paragraphs.get(id) {
                    builder.draw_paragraph(paragraph, to_point(*origin));
                }
            }
            DrawCommand::Image(id, destination) => {
                if let Some(uploaded) = self.resources.textures.get(id) {
                    let source = impellers::Rect::new(
                        impellers::Point::zero(),
                        impellers::Size::new(uploaded.width as f32, uploaded.height as f32),
                    );
                    builder.draw_texture_rect(
                        &uploaded.texture,
                        &source,
                        &to_rect(*destination),
                        TextureSampling::Linear,
                        None,
                    );
                }
            }
        }
    }

    /// Draws with analytic coverage when the shader is loaded and the paint is not blurred;
    /// brushes the shader cannot paint go through a masked layer.
    fn rounded_rect(
        &mut self,
        rect: Rect,
        radii: CornerRadii,
        paint: &Paint,
        stroke: Option<Stroke>,
    ) {
        let shader = self
            .context
            .filter(|_| paint.blur.is_none())
            .and_then(|context| self.resources.shape_shader(context));
        let Some(shader) = shader else {
            let converted = self.msaa_paint(paint, stroke);
            Outline::new(rect, radii, self.corners).draw(&mut self.builder, &converted);
            return;
        };
        let shape = SdfShape::new(rect, radii, self.corners);
        let stroke_width = stroke.map_or(0.0, |stroke| stroke.width);
        if !shape.draw(&mut self.builder, &paint.brush, stroke_width, shader) {
            let converted = fill_paint(paint, self.resources);
            shape.draw_masked(&mut self.builder, &converted, stroke_width, shader);
        }
    }

    fn path(&mut self, path: &Path, paint: &Paint, stroke: Option<Stroke>) {
        if !self.sdf_path(path, paint, stroke) {
            let converted = self.msaa_paint(paint, stroke);
            self.builder.draw_path(&to_path(path), &converted);
        }
    }

    /// Draws a path small on screen with analytic coverage when the shader is loaded and the
    /// paint is not blurred; false leaves it to MSAA. Brushes the shader cannot paint go through
    /// a masked layer.
    fn sdf_path(&mut self, path: &Path, paint: &Paint, stroke: Option<Stroke>) -> bool {
        let Some(context) = self.context.filter(|_| paint.blur.is_none()) else {
            return false;
        };
        let Some(shape) = SdfPath::new(path.bounds(), stroke, self.device_scale) else {
            return false;
        };
        let DrawResources {
            path_program: Some(program),
            segment_textures,
            ..
        } = &mut *self.resources
        else {
            return false;
        };
        let Some(segments) = segment_textures.get_or_upload(context, path, shape.flattening())
        else {
            return false;
        };
        let shader = PathShader::new(context, program);
        match paint.brush {
            Brush::Solid(color) => {
                shape.draw(&mut self.builder, &shape.paint(color, segments, shader));
            }
            _ => {
                let mask = shape.paint(WHITE, segments, shader);
                let converted = fill_paint(paint, self.resources);
                shape.draw_masked(&mut self.builder, &converted, mask);
            }
        }
        true
    }

    fn msaa_paint(&self, paint: &Paint, stroke: Option<Stroke>) -> impellers::Paint {
        match stroke {
            Some(stroke) => stroke_paint(paint, stroke, self.resources),
            None => fill_paint(paint, self.resources),
        }
    }
}

#[cfg(test)]
mod tests {
    use geometry::{CornerRadii, Point, Rect, Transform, Vector, rgb, rgba};
    use graphics::{
        Brush, DrawList, GradientStop, ImageId, Paint, Path, Stroke, TextLayoutId, TextStyle,
        TextSystem,
    };

    use super::*;
    use crate::ImpellerTextSystem;

    fn every_command(text: TextLayoutId) -> DrawList {
        let card = Rect::from_xywh(10.0, 10.0, 200.0, 120.0);
        let radii = CornerRadii::uniform(16.0);
        let gradient = Paint {
            brush: Brush::LinearGradient {
                start: Point::new(0.0, 0.0),
                end: Point::new(200.0, 0.0),
                stops: [
                    GradientStop {
                        offset: 0.0,
                        color: rgb(0x3b82f6),
                    },
                    GradientStop {
                        offset: 1.0,
                        color: rgb(0x9333ea),
                    },
                ]
                .into_iter()
                .collect(),
            },
            blur: None,
        };
        let blurred = Paint {
            blur: Some(6.0),
            ..Paint::solid(rgba(0xffffff80))
        };
        let triangle = Path::new()
            .move_to(Point::new(0.0, 0.0))
            .line_to(Point::new(40.0, 0.0))
            .quad_to(Point::new(50.0, 20.0), Point::new(20.0, 40.0))
            .cubic_to(
                Point::new(10.0, 30.0),
                Point::new(5.0, 20.0),
                Point::new(0.0, 0.0),
            )
            .close();
        let mut list = DrawList::new();
        list.save();
        list.push(DrawCommand::Transform(Transform::rotate(0.1)));
        list.translate(Vector::new(4.0, 4.0));
        list.push(DrawCommand::ClipRect(card));
        list.push(DrawCommand::ClipRoundedRect(card, radii));
        list.shadow(card, radii, rgba(0x00000066), 24.0, Vector::new(0.0, 8.0));
        list.push(DrawCommand::FillRect(card, gradient.clone()));
        list.push(DrawCommand::FillRoundedRect(card, radii, blurred.clone()));
        list.push(DrawCommand::StrokeRoundedRect(
            card,
            radii,
            Stroke::new(2.0),
            gradient,
        ));
        list.push(DrawCommand::FillPath(triangle.clone(), blurred));
        list.push(DrawCommand::StrokePath(
            triangle,
            Stroke::new(3.0),
            Paint::solid(rgb(0xffffff)),
        ));
        list.text(text, Point::new(20.0, 20.0));
        list.text(TextLayoutId(404), Point::new(20.0, 40.0));
        list.push(DrawCommand::Image(ImageId(7), card));
        list.push(DrawCommand::SaveLayer {
            bounds: Some(card),
            opacity: 0.5,
            backdrop_blur: Some(20.0),
        });
        list.push(DrawCommand::SaveLayer {
            bounds: None,
            opacity: 1.0,
            backdrop_blur: None,
        });
        list.restore();
        list.restore();
        list.restore();
        list.restore();
        list.push(DrawCommand::FillRect(
            card,
            Paint {
                brush: Brush::LinearGradient {
                    start: Point::new(0.0, 0.0),
                    end: Point::new(1.0, 1.0),
                    stops: Default::default(),
                },
                blur: None,
            },
        ));
        list
    }

    #[test]
    fn every_draw_command_translates_without_a_gpu_context() {
        let mut text = ImpellerTextSystem::new();
        let layout = text.layout("Hello CrownUI", &TextStyle::default(), None);
        let mut resources = DrawResources::default();
        let Some(paragraph) = layout.into_paragraph() else {
            panic!("Impeller shapes plain text");
        };
        resources.paragraphs.insert(TextLayoutId(1), paragraph);
        for corners in [CornerShape::Round, CornerShape::Squircle] {
            let list = every_command(TextLayoutId(1));
            assert!(translate_draw_list(&list, &mut resources, None, corners, 2.0).is_some());
            let empty = DrawList::new();
            assert!(translate_draw_list(&empty, &mut resources, None, corners, 2.0).is_some());
        }
    }
}
