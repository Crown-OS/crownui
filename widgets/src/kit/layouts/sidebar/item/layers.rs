use geometry::{CornerRadii, Insets, Point, Rect};
use reactive::{Memo, Runtime, Signal};
use view::{Element, Styled, View, bind};

use super::variant::ROW_RADIUS;
use crate::{
    canvas::canvas,
    kit::{Theme, icon::tinted_icon},
    stack::{StackKind, vstack, zstack},
    svg::SvgKind,
    util::{spring, themed::themed},
};

const FILL_INSET: f32 = 0.5;
const BORDER_WIDTH: f32 = 1.0;

fn opacity_of(active: bool) -> f32 {
    if active { 1.0 } else { 0.0 }
}

pub(super) fn hover_wash(
    theme: Signal<Theme>,
    mut visible: impl FnMut(&mut Runtime) -> bool + 'static,
) -> Element<StackKind<()>> {
    vstack(())
        .absolute()
        .inset(FILL_INSET)
        .rounded(ROW_RADIUS)
        .bg(themed(theme, |theme| theme.sidebar.hover_bg))
        .opacity(bind(move |runtime| opacity_of(visible(runtime))))
        .transition(spring::snappy())
}

pub(super) fn selected_pill(theme: Signal<Theme>, selected: Memo<bool>) -> impl View {
    let fill = vstack(())
        .absolute()
        .inset(FILL_INSET)
        .rounded(ROW_RADIUS)
        .bg(themed(theme, |theme| theme.sidebar.selected_bg));
    let border = canvas(move |runtime, bounds, list| {
        let color = theme.with(runtime, |theme| theme.sidebar.selected_border);
        if color.a > 0.0 {
            list.stroke(
                Rect::new(Point::ZERO, bounds).inset(Insets::uniform(BORDER_WIDTH / 2.0)),
                CornerRadii::uniform(ROW_RADIUS),
                BORDER_WIDTH,
                color,
            );
        }
    })
    .absolute()
    .inset(0.0);
    vstack((fill, border))
        .absolute()
        .inset(0.0)
        .opacity(bind(move |runtime| opacity_of(selected.get(runtime))))
        .transition(spring::snappy())
}

pub(super) fn leading_icon(
    theme: Signal<Theme>,
    body: &'static str,
    size: f32,
    stroke: f32,
    selected: Memo<bool>,
) -> Element<StackKind<(Element<SvgKind>, Element<SvgKind>)>> {
    let idle = tinted_icon(body, stroke, themed(theme, |theme| theme.sidebar.icon))
        .size(size)
        .opacity(bind(move |runtime| opacity_of(!selected.get(runtime))))
        .transition(spring::snappy());
    let active = tinted_icon(
        body,
        stroke,
        themed(theme, |theme| theme.sidebar.selected_icon),
    )
    .size(size)
    .opacity(bind(move |runtime| opacity_of(selected.get(runtime))))
    .transition(spring::snappy());
    zstack((idle, active)).size(size).shrink(0.0)
}

pub(super) fn fixed_icon(
    theme: Signal<Theme>,
    body: &'static str,
    size: f32,
    stroke: f32,
    pick: fn(&Theme) -> geometry::Color,
) -> Element<SvgKind> {
    tinted_icon(body, stroke, themed(theme, pick))
        .size(size)
        .shrink(0.0)
}
