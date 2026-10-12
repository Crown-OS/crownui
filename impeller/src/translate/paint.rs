use graphics::{Brush, Paint, Stroke, StrokeCap, StrokeJoin};
use impellers::{BlurStyle, ColorSource, DrawStyle, MaskFilter, Matrix, TextureSampling, TileMode};

use crate::{
    resources::DrawResources,
    util::{
        blur::filter_sigma,
        geometry::{to_color, to_point},
    },
};

pub(crate) fn fill_paint(paint: &Paint, resources: &DrawResources) -> impellers::Paint {
    let mut converted = impellers::Paint::default();
    match &paint.brush {
        Brush::Solid(color) => {
            converted.set_color(to_color(*color));
        }
        Brush::LinearGradient { stops, .. } if stops.is_empty() => {
            converted.set_color(impellers::Color::TRANSPARENT);
        }
        Brush::LinearGradient { start, end, stops } => {
            let colors: Vec<_> = stops.iter().map(|stop| to_color(stop.color)).collect();
            let offsets: Vec<_> = stops.iter().map(|stop| stop.offset).collect();
            let gradient = ColorSource::new_linear_gradient(
                to_point(*start),
                to_point(*end),
                &colors,
                &offsets,
                TileMode::Clamp,
                None,
            );
            converted.set_color_source(&gradient);
        }
        Brush::Image { image, rect } => match resources.textures.get(image) {
            Some(uploaded) if uploaded.width > 0 && uploaded.height > 0 => {
                let placement = Matrix::new_2d(
                    rect.size.width / uploaded.width as f32,
                    0.0,
                    0.0,
                    rect.size.height / uploaded.height as f32,
                    rect.origin.x,
                    rect.origin.y,
                );
                let source = ColorSource::new_image(
                    &uploaded.texture,
                    TileMode::Clamp,
                    TileMode::Clamp,
                    TextureSampling::Linear,
                    Some(&placement),
                );
                converted.set_color_source(&source);
            }
            _ => {
                converted.set_color(impellers::Color::TRANSPARENT);
            }
        },
    }
    if let Some(radius) = paint.blur.filter(|radius| *radius > 0.0) {
        converted.set_mask_filter(&MaskFilter::new_blur(
            BlurStyle::Normal,
            filter_sigma(radius),
        ));
    }
    converted
}

pub(crate) fn stroke_paint(
    paint: &Paint,
    stroke: Stroke,
    resources: &DrawResources,
) -> impellers::Paint {
    let mut converted = fill_paint(paint, resources);
    converted
        .set_draw_style(DrawStyle::Stroke)
        .set_stroke_width(stroke.width)
        .set_stroke_cap(match stroke.cap {
            StrokeCap::Butt => impellers::StrokeCap::Butt,
            StrokeCap::Round => impellers::StrokeCap::Round,
            StrokeCap::Square => impellers::StrokeCap::Square,
        })
        .set_stroke_join(match stroke.join {
            StrokeJoin::Miter => impellers::StrokeJoin::Miter,
            StrokeJoin::Round => impellers::StrokeJoin::Round,
            StrokeJoin::Bevel => impellers::StrokeJoin::Bevel,
        });
    converted
}

pub(crate) fn opacity_paint(opacity: f32) -> impellers::Paint {
    let mut converted = impellers::Paint::default();
    converted.set_color(impellers::Color::new_srgba(
        0.0,
        0.0,
        0.0,
        opacity.clamp(0.0, 1.0),
    ));
    converted
}
