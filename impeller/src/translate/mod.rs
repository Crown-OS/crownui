mod layer;
mod paint;

use geometry::CornerShape;
use graphics::{DrawCommand, DrawList};
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
    },
};

/// Records `list` as an Impeller display list, resolving text and images through `resources`
/// and drawing rounded corners as `corners`; commands naming missing resources are skipped and
/// unbalanced restores are ignored.
pub(crate) fn translate_draw_list(
    list: &DrawList,
    resources: &DrawResources,
    corners: CornerShape,
) -> Option<DisplayList> {
    let mut recorder = Recorder {
        builder: DisplayListBuilder::new(None),
        open_saves: 0,
        resources,
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
                Outline::new(*rect, *radii, self.corners)
                    .draw(builder, &fill_paint(paint, self.resources));
            }
            DrawCommand::StrokeRoundedRect(rect, radii, stroke, paint) => {
                Outline::new(*rect, *radii, self.corners)
                    .draw(builder, &stroke_paint(paint, *stroke, self.resources));
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
            } => {
                let mut paint = impellers::Paint::default();
                paint.set_color(to_color(*color));
                if *blur > 0.0 {
                    paint.set_mask_filter(&MaskFilter::new_blur(
                        BlurStyle::Normal,
                        shadow_sigma(*blur),
                    ));
                }
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
                translate_draw_list(&every_command(TextLayoutId(1)), &resources, corners).is_some()
            );
            assert!(translate_draw_list(&DrawList::new(), &resources, corners).is_some());
        }
    }
}
