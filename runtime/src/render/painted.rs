use geometry::{CornerRadii, Insets, Point, Rect, Size, Transform};
use graphics::{DrawCommand, DrawList};

/// How far a rounded corner reaches along its edges, in radii: continuous (squircle) corners
/// leave the straight edge about 1.53 radii before the corner. The shape is convex, so everything
/// a corner cuts lies on the corner's side of the line between those two points.
const CORNER_REACH: f32 = 1.6;

/// Where something draws, in one coordinate space: what decides whether a rounded clip around it
/// cuts anything a plain rectangle would not.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Painted {
    Nothing,
    Within(Area),
    /// Somewhere this cannot bound, e.g. a canvas that draws through a transform it cannot map.
    Anywhere,
}

/// A bounding box, and for each of its corners (top-left, top-right, bottom-right, bottom-left)
/// the least horizontal plus vertical distance from that corner to anything drawn: a page of rows
/// and full-width dividers has a wide box whose corners are still empty.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Area {
    bounds: Rect,
    gaps: [f32; 4],
}

impl Area {
    fn filled(bounds: Rect) -> Self {
        Self {
            bounds,
            gaps: [0.0; 4],
        }
    }

    fn corners(rect: Rect) -> [Point; 4] {
        [
            Point::new(rect.min_x(), rect.min_y()),
            Point::new(rect.max_x(), rect.min_y()),
            Point::new(rect.max_x(), rect.max_y()),
            Point::new(rect.min_x(), rect.max_y()),
        ]
    }

    /// The gaps measured from the corners of `bounds`, a box around this one.
    fn gaps_from(self, bounds: Rect) -> [f32; 4] {
        let inner = Self::corners(self.bounds);
        let outer = Self::corners(bounds);
        std::array::from_fn(|corner| {
            self.gaps[corner]
                + (inner[corner].x - outer[corner].x).abs()
                + (inner[corner].y - outer[corner].y).abs()
        })
    }
}

impl Painted {
    pub(crate) fn rect(rect: Rect) -> Self {
        Self::Within(Area::filled(rect))
    }

    /// The area `list` covers, with text taken to fill `text_area`, the box it was laid out in.
    pub(crate) fn of_list(list: &DrawList, text_area: Rect) -> Self {
        let mut painted = Self::Nothing;
        let mut transform = Transform::IDENTITY;
        let mut saved = Vec::new();
        for command in list.commands() {
            let area = match command {
                DrawCommand::Save | DrawCommand::SaveLayer { .. } => {
                    saved.push(transform);
                    continue;
                }
                DrawCommand::Restore => {
                    transform = saved.pop().unwrap_or(Transform::IDENTITY);
                    continue;
                }
                DrawCommand::Transform(next) => {
                    transform = next.then(transform);
                    continue;
                }
                DrawCommand::ClipRect(_) | DrawCommand::ClipRoundedRect(..) => continue,
                DrawCommand::FillRect(rect, _) | DrawCommand::FillRoundedRect(rect, _, _) => *rect,
                DrawCommand::StrokeRoundedRect(rect, _, stroke, _) => outset(*rect, stroke.width),
                DrawCommand::FillPath(path, _) => path.bounds(),
                DrawCommand::StrokePath(path, stroke, _) => outset(path.bounds(), stroke.width),
                DrawCommand::Shadow {
                    rect, blur, offset, ..
                } => outset(rect.translate(*offset), *blur * 2.0),
                DrawCommand::Text(_, origin) => text_area.union(Rect::new(*origin, text_area.size)),
                DrawCommand::Image(_, dest) => *dest,
            };
            painted = painted.union(Self::rect(area).transformed(transform));
        }
        painted
    }

    pub(crate) fn union(self, other: Self) -> Self {
        match (self, other) {
            (Self::Anywhere, _) | (_, Self::Anywhere) => Self::Anywhere,
            (Self::Nothing, painted) | (painted, Self::Nothing) => painted,
            (Self::Within(first), Self::Within(second)) => {
                let bounds = first.bounds.union(second.bounds);
                let (first_gaps, second_gaps) = (first.gaps_from(bounds), second.gaps_from(bounds));
                Self::Within(Area {
                    bounds,
                    gaps: std::array::from_fn(|corner| first_gaps[corner].min(second_gaps[corner])),
                })
            }
        }
    }

    /// This area once moved by `transform`: a translation keeps the gaps, anything else bounds
    /// the moved box.
    pub(crate) fn transformed(self, transform: Transform) -> Self {
        let Self::Within(area) = self else {
            return self;
        };
        let [a, b, c, d, x, y] = transform.matrix;
        if (a, b, c, d) == (1.0, 0.0, 0.0, 1.0) {
            return Self::Within(Area {
                bounds: area.bounds.translate(geometry::Vector::new(x, y)),
                gaps: area.gaps,
            });
        }
        let corners = Area::corners(area.bounds).map(|corner| transform.apply(corner));
        let (min, max) = corners
            .iter()
            .fold((corners[0], corners[0]), |(min, max), corner| {
                (
                    Point::new(min.x.min(corner.x), min.y.min(corner.y)),
                    Point::new(max.x.max(corner.x), max.y.max(corner.y)),
                )
            });
        Self::rect(Rect::new(min, Size::new(max.x - min.x, max.y - min.y)))
    }

    /// What is left visible inside `clip`.
    pub(crate) fn clipped(self, clip: Rect) -> Self {
        match self {
            Self::Nothing => Self::Nothing,
            Self::Anywhere => Self::rect(clip),
            Self::Within(area) => area
                .bounds
                .intersection(clip)
                .map_or(Self::Nothing, |bounds| {
                    let before = Area::corners(area.bounds);
                    let after = Area::corners(bounds);
                    Self::Within(Area {
                        bounds,
                        gaps: std::array::from_fn(|corner| {
                            let moved = (after[corner].x - before[corner].x).abs()
                                + (after[corner].y - before[corner].y).abs();
                            (area.gaps[corner] - moved).max(0.0)
                        }),
                    })
                }),
        }
    }

    /// Whether clipping this to `rect` with `radii` cuts nothing that clipping to the plain
    /// rectangle would keep: nothing reaches into a rounded corner.
    pub(crate) fn clears_corners(self, rect: Rect, radii: CornerRadii) -> bool {
        let Self::Within(area) = self else {
            return self == Self::Nothing;
        };
        let radii = [
            radii.top_left,
            radii.top_right,
            radii.bottom_right,
            radii.bottom_left,
        ];
        let clip = Area::corners(rect);
        let content = Area::corners(area.bounds);
        let inward = [(1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)];
        (0..4).all(|corner| {
            let (along_x, along_y) = inward[corner];
            let dx = (content[corner].x - clip[corner].x) * along_x;
            let dy = (content[corner].y - clip[corner].y) * along_y;
            let overflow = (-dx).max(0.0) + (-dy).max(0.0);
            let gap = (area.gaps[corner] - overflow).max(0.0) + dx.max(0.0) + dy.max(0.0);
            radii[corner] <= 0.0 || gap >= radii[corner] * CORNER_REACH
        })
    }

    /// Whether this stays inside `rect`, so clipping to it changes nothing.
    pub(crate) fn inside(self, rect: Rect) -> bool {
        match self {
            Self::Nothing => true,
            Self::Anywhere => false,
            Self::Within(area) => rect.union(area.bounds) == rect,
        }
    }
}

fn outset(rect: Rect, by: f32) -> Rect {
    rect.inset(Insets::uniform(-by / 2.0))
}

#[cfg(test)]
mod tests {
    use geometry::{Vector, rgba};

    use super::*;

    const CARD: Rect = Rect::new(Point::new(0.0, 0.0), Size::new(400.0, 300.0));
    const RADII: CornerRadii = CornerRadii::uniform(12.0);

    #[test]
    fn content_inset_past_the_corners_needs_no_rounded_clip() {
        let label = Painted::rect(Rect::from_xywh(18.0, 14.0, 200.0, 20.0));
        assert!(label.clears_corners(CARD, RADII));
        assert!(!Painted::rect(CARD).clears_corners(CARD, RADII));
        assert!(!Painted::rect(Rect::from_xywh(4.0, 4.0, 50.0, 50.0)).clears_corners(CARD, RADII));
        assert!(!Painted::Anywhere.clears_corners(CARD, RADII));
        assert!(Painted::Nothing.clears_corners(CARD, RADII));
    }

    #[test]
    fn rows_and_full_width_dividers_leave_the_corners_empty() {
        let rows = Painted::rect(Rect::from_xywh(18.0, 14.0, 364.0, 272.0));
        let dividers = Painted::rect(Rect::from_xywh(0.0, 60.0, 400.0, 180.0));
        let card = rows.union(dividers);
        assert!(card.clears_corners(CARD, RADII));
        assert!(!card.inside(Rect::from_xywh(0.0, 0.0, 400.0, 100.0)));
    }

    #[test]
    fn content_scrolled_past_a_corner_reaches_into_it() {
        let rows = Painted::rect(Rect::from_xywh(18.0, -400.0, 364.0, 900.0));
        assert!(!rows.clears_corners(CARD, RADII));
        let clipped = rows.clipped(CARD);
        assert!(clipped.inside(CARD));
        assert!(!clipped.clears_corners(CARD, RADII));
    }

    #[test]
    fn transformed_areas_are_bounded_after_the_move() {
        let area = Painted::rect(Rect::from_xywh(0.0, 0.0, 10.0, 10.0));
        let moved = area.transformed(Transform::translate(Vector::new(5.0, -5.0)));
        assert_eq!(moved, Painted::rect(Rect::from_xywh(5.0, -5.0, 10.0, 10.0)));
        assert!(moved.clipped(CARD).inside(CARD));
    }

    #[test]
    fn a_list_covers_its_fills_strokes_and_shadows() {
        let mut list = DrawList::new();
        list.fill(
            Rect::from_xywh(10.0, 10.0, 20.0, 20.0),
            CornerRadii::ZERO,
            rgba(0xff0000ff),
        );
        list.shadow(
            Rect::from_xywh(10.0, 10.0, 20.0, 20.0),
            CornerRadii::ZERO,
            rgba(0x00000080),
            4.0,
            Vector::new(0.0, 2.0),
        );
        let painted = Painted::of_list(&list, Rect::ZERO);
        let Painted::Within(area) = painted else {
            panic!("the list draws");
        };
        assert_eq!(area.bounds, Rect::from_xywh(6.0, 8.0, 28.0, 28.0));
    }
}
