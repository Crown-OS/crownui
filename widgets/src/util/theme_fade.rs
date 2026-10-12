use motion::Curve;

const THEME_FADE_MILLIS: u64 = 240;

/// The curve a palette change fades along: crownuikit's 240 ms smoothstep cross-fade.
pub(crate) fn theme_fade() -> Curve {
    Curve::ease_in_out(THEME_FADE_MILLIS)
}
