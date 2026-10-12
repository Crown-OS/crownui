use geometry::{Color, Point, Size};
use graphics::{DrawCommand, DrawList, Paint, Path, PathCommand, Stroke, StrokeCap, StrokeJoin};
use semantics::Role;
use usvg::tiny_skia_path::PathSegment;
use view::{Cx, Element, IntoProp, Kind, NodeId, Prop};

/// One flattened, absolutely transformed SVG shape.
#[derive(Debug, Clone, PartialEq)]
struct SvgShape {
    path: Path,
    fill: Option<Color>,
    stroke: Option<(Stroke, Color)>,
}

/// A parsed SVG ready to draw at any size.
#[derive(Debug, Clone, PartialEq)]
struct SvgPicture {
    size: Size,
    shapes: Vec<SvgShape>,
}

fn to_color(color: usvg::Color, opacity: f32) -> Color {
    Color::from_rgba8(
        color.red,
        color.green,
        color.blue,
        (opacity.clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

fn convert_path(path: &usvg::Path) -> Option<SvgShape> {
    let transform = path.abs_transform();
    let map = |point: usvg::tiny_skia_path::Point| {
        let mut mapped = point;
        transform.map_point(&mut mapped);
        Point::new(mapped.x, mapped.y)
    };
    let commands = path
        .data()
        .segments()
        .map(|segment| match segment {
            PathSegment::MoveTo(point) => PathCommand::MoveTo(map(point)),
            PathSegment::LineTo(point) => PathCommand::LineTo(map(point)),
            PathSegment::QuadTo(control, point) => PathCommand::QuadTo(map(control), map(point)),
            PathSegment::CubicTo(first, second, point) => {
                PathCommand::CubicTo(map(first), map(second), map(point))
            }
            PathSegment::Close => PathCommand::Close,
        })
        .collect();
    let solid = |paint: &usvg::Paint, opacity: f32| match paint {
        usvg::Paint::Color(color) => Some(to_color(*color, opacity)),
        _ => None,
    };
    let fill = path
        .fill()
        .and_then(|fill| solid(fill.paint(), fill.opacity().get()));
    let scale = (transform.sx.abs() + transform.sy.abs()) / 2.0;
    let stroke = path.stroke().and_then(|stroke| {
        let outline = Stroke {
            width: stroke.width().get() * scale,
            cap: match stroke.linecap() {
                usvg::LineCap::Butt => StrokeCap::Butt,
                usvg::LineCap::Round => StrokeCap::Round,
                usvg::LineCap::Square => StrokeCap::Square,
            },
            join: match stroke.linejoin() {
                usvg::LineJoin::Round => StrokeJoin::Round,
                usvg::LineJoin::Bevel => StrokeJoin::Bevel,
                usvg::LineJoin::Miter | usvg::LineJoin::MiterClip => StrokeJoin::Miter,
            },
        };
        solid(stroke.paint(), stroke.opacity().get()).map(|color| (outline, color))
    });
    (fill.is_some() || stroke.is_some()).then_some(SvgShape {
        path: Path { commands },
        fill,
        stroke,
    })
}

fn collect(group: &usvg::Group, shapes: &mut Vec<SvgShape>) {
    for node in group.children() {
        match node {
            usvg::Node::Group(group) => collect(group, shapes),
            usvg::Node::Path(path) => shapes.extend(convert_path(path)),
            usvg::Node::Image(_) | usvg::Node::Text(_) => {}
        }
    }
}

fn parse(source: &str) -> Result<SvgPicture, usvg::Error> {
    let tree = usvg::Tree::from_str(source, &usvg::Options::default())?;
    let mut shapes = Vec::new();
    collect(tree.root(), &mut shapes);
    Ok(SvgPicture {
        size: Size::new(tree.size().width(), tree.size().height()),
        shapes,
    })
}

fn tinted(color: Color, tint: Option<Color>) -> Color {
    tint.map_or(color, |tint| tint.with_alpha_factor(color.a))
}

fn draw(picture: &SvgPicture, tint: Option<Color>, size: Size, list: &mut DrawList) {
    if picture.size.is_empty() || size.is_empty() {
        return;
    }
    let scale = (size.width / picture.size.width).min(size.height / picture.size.height);
    let offset = geometry::Vector::new(
        (size.width - picture.size.width * scale) / 2.0,
        (size.height - picture.size.height * scale) / 2.0,
    );
    list.save();
    list.translate(offset);
    list.transform(geometry::Transform::scale(scale, scale));
    for shape in &picture.shapes {
        if let Some(fill) = shape.fill {
            list.push(DrawCommand::FillPath(
                shape.path.clone(),
                Paint::solid(tinted(fill, tint)),
            ));
        }
        if let Some((outline, color)) = shape.stroke {
            list.push(DrawCommand::StrokePath(
                shape.path.clone(),
                outline,
                Paint::solid(tinted(color, tint)),
            ));
        }
    }
    list.restore();
}

/// The kind behind [`svg`].
pub struct SvgKind {
    source: String,
    tint: Option<Prop<Color>>,
}

impl Kind for SvgKind {
    const NAME: &'static str = "svg";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        mount_svg(cx, node, &self.source, self.tint);
    }
}

/// Draws `source` into `node`, tinted by `tint` (re-drawn whenever a signal it reads changes).
pub(crate) fn mount_svg(cx: &mut Cx, node: NodeId, source: &str, mut tint: Option<Prop<Color>>) {
    cx.set_role(node, Role::Image);
    let picture = match parse(source) {
        Ok(picture) => picture,
        Err(error) => {
            tracing::warn!("CUI0501: invalid SVG: {error}");
            return;
        }
    };
    let natural = picture.size;
    cx.set_default_style(
        node,
        view::StyleProp::Size(
            view::SizeProperty::Width,
            layouting::Dimension::Length(natural.width),
        ),
    );
    cx.set_default_style(
        node,
        view::StyleProp::Size(
            view::SizeProperty::Height,
            layouting::Dimension::Length(natural.height),
        ),
    );
    cx.set_canvas(node, move |runtime, size, list| {
        let tint = match tint.as_mut() {
            Some(Prop::Static(color)) => Some(*color),
            Some(Prop::Dynamic(compute)) => Some(compute(runtime)),
            None => None,
        };
        draw(&picture, tint, size, list);
    });
}

/// Vector artwork from SVG source, scaled to the element's size (natural size by default).
/// Solid fills and strokes are supported; gradients, images and text inside SVGs are skipped.
pub fn svg(source: impl Into<String>) -> Element<SvgKind> {
    Element::new(SvgKind {
        source: source.into(),
        tint: None,
    })
}

/// An SVG icon whose colors are replaced by `tint` (keeping each shape's alpha), like Lucide
/// icons drawn with `currentColor`.
pub fn icon(source: impl Into<String>, tint: impl IntoProp<Color>) -> Element<SvgKind> {
    Element::new(SvgKind {
        source: source.into(),
        tint: Some(tint.into_prop()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SQUARE: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><rect x="2" y="2" width="20" height="20" fill="#ff0000" stroke="#000000" stroke-width="2"/></svg>"##;

    #[test]
    fn shapes_are_flattened_with_fill_and_stroke() {
        let Ok(picture) = parse(SQUARE) else {
            panic!("valid svg");
        };
        assert_eq!(picture.size, Size::new(24.0, 24.0));
        assert_eq!(picture.shapes.len(), 1);
        assert_eq!(picture.shapes[0].fill, Some(geometry::rgb(0xff0000)));
        assert_eq!(
            picture.shapes[0].stroke.map(|(outline, _)| outline.width),
            Some(2.0)
        );
    }

    #[test]
    fn drawing_scales_to_fit_and_centers() {
        let Ok(picture) = parse(SQUARE) else {
            panic!("valid svg");
        };
        let mut list = DrawList::new();
        draw(
            &picture,
            Some(geometry::WHITE),
            Size::new(48.0, 24.0),
            &mut list,
        );
        let text = list.to_string();
        assert!(
            text.contains("transform [1.0, 0.0, 0.0, 1.0, 12.0, 0.0]"),
            "{text}"
        );
        assert!(text.contains("#ffffffff"), "{text}");
    }
}
