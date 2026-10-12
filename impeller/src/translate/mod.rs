mod layer;
mod paint;

use geometry::CornerShape;
use geometry::{CornerRadii, Rect};
use graphics::{DrawCommand, DrawList, Paint, Stroke};
use impellers::{
    BlurStyle, ClipOperation, DisplayList, DisplayListBuilder, MaskFilter, TextureSampling,
};

pub(crate) use layer::save_layer;
use paint::{fill_paint, stroke_paint};

use crate::{
    resources::DrawResources,
    util::{
        blur::shadow_sigma,
        geometry::{to_color, to_matrix, to_path, to_point, to_rect},
        outline::Outline,
        sdf_shape::SdfShape,
        shape_shader::ShapeShader,
    },
};

/// Records `list` as an Impeller display list, resolving text and images through `resources`
/// and drawing rounded corners as `corners`, antialiased by `shader` when there is one (MSAA
/// otherwise); commands naming missing resources are skipped and unbalanced restores ignored.
pub(crate) fn translate_draw_list(
    list: &DrawList,
    resources: &DrawResources,
    shader: Option<ShapeShader<'_>>,
    corners: CornerShape,
) -> Option<DisplayList> {
    let mut recorder = Recorder {
        builder: DisplayListBuilder::new(None),
        open_saves: 0,
        resources,
        shader,
        corners,
    };
    for command in list.commands() {
        recorder.record(command);
    }
    recorder.builder.build()
}

struct Recorder<'resources> {
    builder: DisplayListBuilder,
    open_saves: usize,
    resources: &'resources DrawResources,
    shader: Option<ShapeShader<'resources>>,
    corners: CornerShape,
}

impl Recorder<'_> {
    fn record(&mut self, command: &DrawCommand) {
        let builder = &mut self.builder;
        match command {
            DrawCommand::Save => {
                builder.save();
                self.open_saves += 1;
            }
            DrawCommand::Restore => {
                if self.open_saves > 0 {
                    builder.restore();
                    self.open_saves -= 1;
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
                self.open_saves += 1;
            }
            DrawCommand::Transform(transform) => {
                builder.transform(&to_matrix(*transform));
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
                builder.draw_path(&to_path(path), &fill_paint(paint, self.resources));
            }
            DrawCommand::StrokePath(path, stroke, paint) => {
                builder.draw_path(
                    &to_path(path),
                    &stroke_paint(paint, *stroke, self.resources),
                );
            }
            DrawCommand::Shadow {
                rect,
                radii,
                color,
                blur,
                offset,
            } if *blur <= 0.0 => {
                self.rounded_rect(rect.translate(*offset), *radii, &Paint::solid(*color), None);
            }
            DrawCommand::Shadow {
                rect,
                radii,
                color,
                blur,
                offset,
            } => {
                let mut paint = impellers::Paint::default();
                paint.set_color(to_color(*color));
                paint.set_mask_filter(&MaskFilter::new_blur(
                    BlurStyle::Normal,
                    shadow_sigma(*blur),
                ));
                Outline::new(rect.translate(*offset), *radii, self.corners).draw(builder, &paint);
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
        let Some(shader) = self.shader.filter(|_| paint.blur.is_none()) else {
            let converted = match stroke {
                Some(stroke) => stroke_paint(paint, stroke, self.resources),
                None => fill_paint(paint, self.resources),
            };
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
            assert!(
                translate_draw_list(&every_command(TextLayoutId(1)), &resources, None, corners)
                    .is_some()
            );
            assert!(translate_draw_list(&DrawList::new(), &resources, None, corners).is_some());
        }
    }
}
