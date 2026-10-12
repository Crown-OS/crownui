/// The curve a rounded corner follows, named after CSS `corner-shape`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CornerShape {
    /// A circular arc of the corner's radius.
    #[default]
    Round,
    /// The continuous corner CrownOS's compositor draws: a quarter of `x⁴ + y⁴ = 1` spread over
    /// one and a half times the radius, so the curvature eases in from the edge instead of
    /// jumping to a circle's.
    Squircle,
}
