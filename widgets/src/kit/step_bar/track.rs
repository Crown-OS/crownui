use geometry::{CornerRadii, Point, Rect, Size, Vector};
use reactive::{Memo, Runtime, Signal};
use view::{Styled, View, bind};

use super::{TRACK_GAP, TRACK_HEIGHT};
use crate::{
    canvas::canvas,
    kit::Theme,
    stack::vstack,
    util::{spring, themed::themed},
};

const TRACK_RADIUS: f32 = TRACK_HEIGHT / 2.0;

fn track_width(bar: Size, total: usize) -> f32 {
    let gaps = TRACK_GAP * total.saturating_sub(1) as f32;
    ((bar.width - gaps) / total as f32).max(0.0)
}

pub(super) fn track(
    theme: Signal<Theme>,
    filled: Memo<f32>,
    bar: Signal<Size>,
    total: usize,
    index: usize,
) -> impl View {
    let unfilled_width = move |runtime: &mut Runtime| {
        let fraction = (filled.get(runtime) - index as f32).clamp(0.0, 1.0);
        (1.0 - fraction) * bar.with(runtime, |bar| track_width(*bar, total))
    };
    let gradient = canvas(move |runtime, bounds, list| {
        let accent = theme.with(runtime, |theme| theme.accent);
        list.fill(
            Rect::new(Point::ZERO, bounds),
            CornerRadii::uniform(TRACK_RADIUS),
            accent.horizontal(0.0, bounds.width),
        );
    })
    .absolute()
    .inset(0.0)
    .translate(bind(move |runtime| {
        Vector::new(unfilled_width(runtime), 0.0)
    }));
    let wipe = vstack(gradient)
        .absolute()
        .inset(0.0)
        .rounded(TRACK_RADIUS)
        .overflow_hidden()
        .translate(bind(move |runtime| {
            Vector::new(-unfilled_width(runtime), 0.0)
        }))
        .transition(spring::crisp());
    vstack(wipe)
        .relative()
        .flex_1()
        .h(TRACK_HEIGHT)
        .rounded(TRACK_RADIUS)
        .overflow_hidden()
        .bg(themed(theme, |theme| theme.control.track))
}
