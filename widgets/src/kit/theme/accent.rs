use crownconfig::AccentColor;
use geometry::{Color, Point, rgb};
use graphics::{Brush, GradientStop, Paint};

use crate::util::color::{mix, scale_alpha};

/// The two stops of every accent surface; the first is the lighter end, projected toward the top
/// or the left.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GradientStops {
    /// The lighter stop.
    pub start: Color,
    /// The deeper stop.
    pub end: Color,
}

impl GradientStops {
    /// Stops from `start` to `end`.
    pub const fn new(start: Color, end: Color) -> Self {
        Self { start, end }
    }

    /// The same color at both ends: a flat fill wearing the gradient's shape.
    pub const fn flat(color: Color) -> Self {
        Self::new(color, color)
    }

    /// Blends stop for stop, so a surface can travel between two gradients.
    pub fn mix(self, target: Self, progress: f32) -> Self {
        Self::new(
            mix(self.start, target.start, progress),
            mix(self.end, target.end, progress),
        )
    }

    /// Applies `change` to both stops.
    pub fn map(self, change: impl Fn(Color) -> Color) -> Self {
        Self::new(change(self.start), change(self.end))
    }

    /// Both stops with their alpha scaled by `factor`.
    pub fn fade(self, factor: f32) -> Self {
        self.map(|color| scale_alpha(color, factor))
    }

    /// A top-to-bottom gradient between `top` and `bottom`, in the painting node's coordinates.
    pub fn vertical(self, top: f32, bottom: f32) -> Paint {
        self.paint(Point::new(0.0, top), Point::new(0.0, bottom))
    }

    /// A left-to-right gradient between `left` and `right`, in the painting node's coordinates.
    pub fn horizontal(self, left: f32, right: f32) -> Paint {
        self.paint(Point::new(left, 0.0), Point::new(right, 0.0))
    }

    fn paint(self, start: Point, end: Point) -> Paint {
        Paint {
            brush: Brush::LinearGradient {
                start,
                end,
                stops: [
                    GradientStop {
                        offset: 0.0,
                        color: self.start,
                    },
                    GradientStop {
                        offset: 1.0,
                        color: self.end,
                    },
                ]
                .into_iter()
                .collect(),
            },
            blur: None,
        }
    }
}

/// The gradient stops each named accent resolves to.
pub const fn accent_stops(accent: AccentColor) -> GradientStops {
    match accent {
        AccentColor::Purple => GradientStops::new(rgb(0x8f6dfb), rgb(0x6d48e8)),
        AccentColor::Blue => GradientStops::new(rgb(0x5aa2ff), rgb(0x2563eb)),
        AccentColor::Green => GradientStops::new(rgb(0x4ade80), rgb(0x16a34a)),
        AccentColor::Orange => GradientStops::new(rgb(0xfb923c), rgb(0xea580c)),
        AccentColor::Pink => GradientStops::new(rgb(0xf472b6), rgb(0xdb2777)),
    }
}

/// The CrownOS purple, the accent before the user picks one.
pub const DEFAULT_ACCENT: GradientStops = accent_stops(AccentColor::Purple);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_named_accent_is_distinct() {
        let all = [
            AccentColor::Purple,
            AccentColor::Blue,
            AccentColor::Green,
            AccentColor::Orange,
            AccentColor::Pink,
        ];
        for (index, first) in all.iter().enumerate() {
            for second in &all[index + 1..] {
                assert_ne!(accent_stops(*first), accent_stops(*second));
            }
        }
    }

    #[test]
    fn mixing_endpoints_return_the_endpoints() {
        let (from, to) = (DEFAULT_ACCENT, accent_stops(AccentColor::Orange));
        assert_eq!(from.mix(to, 0.0), from);
        assert_eq!(from.mix(to, 1.0), to);
    }
}
