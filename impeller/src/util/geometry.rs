use geometry::{Color, CornerRadii, Point, Rect, Transform};
use graphics::{Path, PathCommand};
use impellers::{FillType, Matrix, PathBuilder, RoundingRadii};

pub(crate) fn to_point(point: Point) -> impellers::Point {
    impellers::Point::new(point.x, point.y)
}

pub(crate) fn to_rect(rect: Rect) -> impellers::Rect {
    impellers::Rect::new(
        impellers::Point::new(rect.origin.x, rect.origin.y),
        impellers::Size::new(rect.size.width, rect.size.height),
    )
}

pub(crate) fn to_radii(radii: CornerRadii) -> RoundingRadii {
    let corner = |radius: f32| impellers::Point::new(radius, radius);
    RoundingRadii {
        top_left: corner(radii.top_left),
        top_right: corner(radii.top_right),
        bottom_left: corner(radii.bottom_left),
        bottom_right: corner(radii.bottom_right),
    }
}

pub(crate) fn to_color(color: Color) -> impellers::Color {
    impellers::Color::new_srgba(color.r, color.g, color.b, color.a)
}

pub(crate) fn to_matrix(transform: Transform) -> Matrix {
    let [a, b, c, d, tx, ty] = transform.matrix;
    Matrix::new_2d(a, b, c, d, tx, ty)
}

pub(crate) fn to_path(path: &Path) -> impellers::Path {
    let mut builder = PathBuilder::default();
    for command in &path.commands {
        match *command {
            PathCommand::MoveTo(point) => builder.move_to(to_point(point)),
            PathCommand::LineTo(point) => builder.line_to(to_point(point)),
            PathCommand::QuadTo(control, point) => {
                builder.quadratic_curve_to(to_point(control), to_point(point))
            }
            PathCommand::CubicTo(first, second, point) => {
                builder.cubic_curve_to(to_point(first), to_point(second), to_point(point))
            }
            PathCommand::Close => builder.close(),
        };
    }
    builder.take_path_new(FillType::NonZero)
}

#[cfg(test)]
mod tests {
    use geometry::Vector;

    use super::*;

    #[test]
    fn affine_transforms_map_points_like_geometry_does() {
        let transform = Transform::rotate(0.5).then(Transform::translate(Vector::new(10.0, -4.0)));
        let point = Point::new(3.0, 7.0);
        let expected = transform.apply(point);
        let Some(mapped) =
            to_matrix(transform).transform_point2d(impellers::Point::new(point.x, point.y))
        else {
            panic!("affine matrix must map every point");
        };
        assert!((mapped.x - expected.x).abs() < 1e-4);
        assert!((mapped.y - expected.y).abs() < 1e-4);
    }
}
