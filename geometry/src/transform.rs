use crate::{Point, Vector};

/// A 2D affine transform `[a c tx; b d ty]` applied to column vectors.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Transform {
    /// Matrix coefficients in the order `a, b, c, d, tx, ty`.
    pub matrix: [f32; 6],
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform {
    /// The transform that changes nothing.
    pub const IDENTITY: Self = Self {
        matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    };

    /// A translation by `offset`.
    pub const fn translate(offset: Vector) -> Self {
        Self {
            matrix: [1.0, 0.0, 0.0, 1.0, offset.x, offset.y],
        }
    }

    /// A scale about the origin.
    pub const fn scale(x: f32, y: f32) -> Self {
        Self {
            matrix: [x, 0.0, 0.0, y, 0.0, 0.0],
        }
    }

    /// A clockwise rotation about the origin, in radians.
    pub fn rotate(radians: f32) -> Self {
        let (sin, cos) = radians.sin_cos();
        Self {
            matrix: [cos, sin, -sin, cos, 0.0, 0.0],
        }
    }

    /// `self` followed by `next`.
    pub fn then(self, next: Self) -> Self {
        let [a1, b1, c1, d1, x1, y1] = self.matrix;
        let [a2, b2, c2, d2, x2, y2] = next.matrix;
        Self {
            matrix: [
                a2 * a1 + c2 * b1,
                b2 * a1 + d2 * b1,
                a2 * c1 + c2 * d1,
                b2 * c1 + d2 * d1,
                a2 * x1 + c2 * y1 + x2,
                b2 * x1 + d2 * y1 + y2,
            ],
        }
    }

    /// Applies the transform about `origin` instead of `(0, 0)`.
    pub fn about(self, origin: Point) -> Self {
        let to_origin = origin.to_vector();
        Self::translate(-to_origin)
            .then(self)
            .then(Self::translate(to_origin))
    }

    /// Maps a point.
    pub fn apply(self, point: Point) -> Point {
        let [a, b, c, d, tx, ty] = self.matrix;
        Point::new(
            a * point.x + c * point.y + tx,
            b * point.x + d * point.y + ty,
        )
    }

    /// The inverse, if the transform is not degenerate.
    pub fn invert(self) -> Option<Self> {
        let [a, b, c, d, tx, ty] = self.matrix;
        let determinant = a * d - b * c;
        if determinant.abs() < f32::EPSILON {
            return None;
        }
        let inverse = 1.0 / determinant;
        let (ia, ib, ic, id) = (d * inverse, -b * inverse, -c * inverse, a * inverse);
        Some(Self {
            matrix: [ia, ib, ic, id, -(ia * tx + ic * ty), -(ib * tx + id * ty)],
        })
    }

    /// Whether this is the identity.
    pub fn is_identity(self) -> bool {
        self == Self::IDENTITY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composition_applies_left_to_right() {
        let transform =
            Transform::scale(2.0, 2.0).then(Transform::translate(Vector::new(1.0, 0.0)));
        assert_eq!(transform.apply(Point::new(1.0, 1.0)), Point::new(3.0, 2.0));
    }

    #[test]
    fn inverse_undoes_the_transform() {
        let transform = Transform::rotate(0.5).then(Transform::translate(Vector::new(3.0, -2.0)));
        let Some(inverse) = transform.invert() else {
            panic!("rotation plus translation is invertible");
        };
        let round_trip = inverse.apply(transform.apply(Point::new(4.0, 5.0)));
        assert!((round_trip.x - 4.0).abs() < 1e-4 && (round_trip.y - 5.0).abs() < 1e-4);
    }

    #[test]
    fn scaling_about_a_point_keeps_it_fixed() {
        let center = Point::new(10.0, 10.0);
        assert_eq!(
            Transform::scale(2.0, 2.0).about(center).apply(center),
            center
        );
    }
}
