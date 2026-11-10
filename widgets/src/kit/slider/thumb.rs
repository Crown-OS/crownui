use geometry::{BLACK, CornerRadii, Point, Rect, Vector};
use reactive::Signal;
use view::{IntoProp, Styled, View};

use super::metrics::{
    HEIGHT, THUMB_BACKDROP_BLUR, THUMB_FILL_ALPHA, THUMB_HEIGHT, THUMB_SHADOW_ALPHA,
    THUMB_SHADOW_BLUR, THUMB_SHADOW_OFFSET, THUMB_WIDTH,
};
use crate::{
    canvas::canvas,
    kit::Theme,
    stack::vstack,
    util::{color::scale_alpha, surface_paint::InnerShadow, themed::themed},
};

/// The shade pressed in from the thumb's top-left: tinted by the accent it rides in light mode,
/// plain black in dark mode.
fn inner_shadow(theme: &Theme) -> InnerShadow {
    let tint = if theme.mode.is_dark() {
        BLACK
    } else {
        theme.accent.start
    };
    InnerShadow {
        color: scale_alpha(tint, THUMB_SHADOW_ALPHA),
        offset: THUMB_SHADOW_OFFSET,
        blur: THUMB_SHADOW_BLUR,
    }
}

pub(super) fn thumb(theme: Signal<Theme>, travel: impl IntoProp<Vector>) -> impl View {
    let shade = canvas(move |runtime, bounds, list| {
        theme.with(runtime, inner_shadow).paint(
            list,
            Rect::new(Point::ZERO, bounds),
            CornerRadii::uniform(bounds.height / 2.0),
        );
    })
    .absolute()
    .inset(0.0);
    vstack(shade)
        .absolute()
        .left(0.0)
        .top((HEIGHT - THUMB_HEIGHT) / 2.0)
        .w(THUMB_WIDTH)
        .h(THUMB_HEIGHT)
        .rounded(THUMB_HEIGHT / 2.0)
        .backdrop_blur(THUMB_BACKDROP_BLUR)
        .bg(themed(theme, |theme| {
            scale_alpha(theme.control.knob, THUMB_FILL_ALPHA)
        }))
        .translate(travel)
}
