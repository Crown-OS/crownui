use geometry::{Color, CornerRadii, Point, Rect, Vector, rgb};
use graphics::{Brush, GradientStop, Paint};
use reactive::Signal;
use view::{IntoProp, Styled, View};

use super::metrics::{HEIGHT, THUMB_FILL_ALPHA, THUMB_HEIGHT, THUMB_RIM_DEPTH, THUMB_WIDTH};
use crate::{
    canvas::canvas,
    kit::Theme,
    stack::vstack,
    util::{color::scale_alpha, themed::themed},
};

const WHITE_SMOKE: Color = rgb(0xf5f5f5);

/// A highlight at each end of the thumb fading to nothing a rim's depth in, filled in the
/// thumb's own capsule so the ends stay round without a clip.
fn rim_highlight(width: f32) -> Paint {
    let fade = WHITE_SMOKE.with_alpha_factor(0.0);
    let depth = (THUMB_RIM_DEPTH / width).min(0.5);
    let stops = [
        (0.0, WHITE_SMOKE),
        (depth, fade),
        (1.0 - depth, fade),
        (1.0, WHITE_SMOKE),
    ];
    Paint {
        brush: Brush::LinearGradient {
            start: Point::ZERO,
            end: Point::new(width, 0.0),
            stops: stops
                .into_iter()
                .map(|(offset, color)| GradientStop { offset, color })
                .collect(),
        },
        blur: None,
    }
}

pub(super) fn thumb(theme: Signal<Theme>, travel: impl IntoProp<Vector>) -> impl View {
    let finish = canvas(move |_, bounds, list| {
        list.fill(
            Rect::new(Point::ZERO, bounds),
            CornerRadii::uniform(bounds.height / 2.0),
            rim_highlight(bounds.width),
        );
    })
    .absolute()
    .inset(0.0);
    vstack(finish)
        .absolute()
        .left(0.0)
        .top((HEIGHT - THUMB_HEIGHT) / 2.0)
        .w(THUMB_WIDTH)
        .h(THUMB_HEIGHT)
        .rounded(THUMB_HEIGHT / 2.0)
        .bg(themed(theme, |theme| {
            scale_alpha(theme.control.knob, THUMB_FILL_ALPHA)
        }))
        .translate(travel)
}
