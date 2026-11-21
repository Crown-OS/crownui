use crate::Point;

/// Smallest homogeneous weight a point keeps: points nearer the eye than this are behind it.
const NEAREST_WEIGHT: f32 = 1.0e-4;

/// A plane turned about its horizontal and then its vertical axis through `pivot`, seen in
/// perspective from `distance` in front of it: CSS `perspective() rotateY() rotateX()`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tilt {
    /// Turn about the horizontal axis, in radians; positive brings the bottom edge nearer.
    pub pitch: f32,
    /// Turn about the vertical axis, in radians; positive takes the right edge away.
    pub yaw: f32,
    /// Distance from the eye to the plane; 0 or less sees it without perspective.
    pub distance: f32,
    /// The point the plane turns about and its far side converges toward.
    pub pivot: Point,
}

impl Tilt {
    /// Whether the plane faces the eye, so the tilt changes nothing.
    pub fn is_flat(self) -> bool {
        self.pitch == 0.0 && self.yaw == 0.0
    }

    /// The projective map as rows producing `x`, `y` and the weight `w` from `(x, y, 1)`.
    pub fn rows(self) -> [[f32; 3]; 3] {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let inverse_distance = if self.distance > 0.0 {
            self.distance.recip()
        } else {
            0.0
        };
        let turned = [
            [cos_yaw, sin_pitch * sin_yaw, 0.0],
            [0.0, cos_pitch, 0.0],
            [
                sin_yaw * inverse_distance,
                -sin_pitch * cos_yaw * inverse_distance,
                1.0,
            ],
        ];
        let Point { x, y } = self.pivot;
        multiply(multiply(translation(x, y), turned), translation(-x, -y))
    }

    /// Maps a point; `None` when it lands behind the eye.
    pub fn apply(self, point: Point) -> Option<Point> {
        let [row_x, row_y, row_w] = self.rows();
        let project = |row: [f32; 3]| row[0] * point.x + row[1] * point.y + row[2];
        let weight = project(row_w);
        (weight > NEAREST_WEIGHT)
            .then(|| Point::new(project(row_x) / weight, project(row_y) / weight))
    }
}

fn translation(x: f32, y: f32) -> [[f32; 3]; 3] {
    [[1.0, 0.0, x], [0.0, 1.0, y], [0.0, 0.0, 1.0]]
}

fn multiply(left: [[f32; 3]; 3], right: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    std::array::from_fn(|row| {
        std::array::from_fn(|column| (0..3).map(|k| left[row][k] * right[k][column]).sum())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const PIVOT: Point = Point::new(50.0, 50.0);

    fn tilt(pitch_degrees: f32) -> Tilt {
        Tilt {
            pitch: pitch_degrees.to_radians(),
            yaw: 0.0,
            distance: 200.0,
            pivot: PIVOT,
        }
    }

    fn mapped(tilt: Tilt, point: Point) -> Point {
        let Some(mapped) = tilt.apply(point) else {
            panic!("{point:?} landed behind the eye");
        };
        mapped
    }

    #[test]
    fn a_flat_tilt_changes_nothing() {
        let flat = tilt(0.0);
        assert!(flat.is_flat());
        assert_eq!(mapped(flat, Point::new(10.0, 90.0)), Point::new(10.0, 90.0));
    }

    #[test]
    fn the_pivot_stays_where_it_is() {
        let point = mapped(tilt(40.0), PIVOT);
        assert!((point.x - PIVOT.x).abs() < 1e-3 && (point.y - PIVOT.y).abs() < 1e-3);
    }

    #[test]
    fn a_positive_pitch_widens_the_bottom_edge_and_narrows_the_top() {
        let tilted = tilt(40.0);
        let width_at =
            |y| mapped(tilted, Point::new(100.0, y)).x - mapped(tilted, Point::new(0.0, y)).x;
        assert!(
            width_at(100.0) > 100.0,
            "bottom edge is {}",
            width_at(100.0)
        );
        assert!(width_at(0.0) < 100.0, "top edge is {}", width_at(0.0));
    }

    #[test]
    fn a_plane_turned_past_the_eye_is_reported() {
        let past = Tilt {
            distance: 10.0,
            ..tilt(80.0)
        };
        assert_eq!(past.apply(Point::new(50.0, 100.0)), None);
    }
}
