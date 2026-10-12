use reactive::{Memo, Signal};
use view::{Element, Styled, bind};

use super::stage::Stage;
use crate::{
    kit::{INTER_FAMILY, TEXT_SIZE, Theme, icon::tinted_icon, icons},
    stack::{StackKind, hstack, vstack},
    svg::SvgKind,
    text::{TextKind, text},
    util::themed::themed,
};

pub(super) const DOT_SIZE: f32 = 24.0;
const RING_WIDTH: f32 = 2.0;
const RING_GAP: f32 = 2.5;
const LABEL_GAP: f32 = 7.0;
const LABEL_TEXT_SIZE: f32 = TEXT_SIZE - 2.0;
const CHECK_STROKE: f32 = 2.0;
const CHECK_INSET: f32 = 6.0;

type Dot = Element<StackKind<(Element<StackKind<()>>, Element<SvgKind>)>>;

fn ring(theme: Signal<Theme>, step: usize, current: Memo<usize>) -> Element<StackKind<()>> {
    let outset = RING_GAP + RING_WIDTH;
    vstack(())
        .absolute()
        .inset(-outset)
        .rounded(DOT_SIZE / 2.0 + outset)
        .border(RING_WIDTH)
        .border_color(themed(theme, |theme| theme.accent.end))
        .hidden(bind(move |runtime| {
            Stage::of(step, current.get(runtime)) != Stage::Current
        }))
}

fn check(theme: Signal<Theme>, step: usize, current: Memo<usize>) -> Element<SvgKind> {
    tinted_icon(
        icons::CHECK,
        CHECK_STROKE,
        themed(theme, |theme| theme.text.on_accent),
    )
    .size(DOT_SIZE - 2.0 * CHECK_INSET)
    .hidden(bind(move |runtime| {
        Stage::of(step, current.get(runtime)) != Stage::Done
    }))
}

fn dot(theme: Signal<Theme>, step: usize, current: Memo<usize>) -> Dot {
    hstack((ring(theme, step, current), check(theme, step, current)))
        .size(DOT_SIZE)
        .rounded(DOT_SIZE / 2.0)
        .items_center()
        .justify_center()
        .shrink(0.0)
        .bg(bind(move |runtime| {
            let stage = Stage::of(step, current.get(runtime));
            theme.with(runtime, |theme| stage.dot_fill(theme))
        }))
}

fn caption(
    theme: Signal<Theme>,
    step: usize,
    current: Memo<usize>,
    label: String,
) -> Element<TextKind> {
    text(label)
        .text_size(LABEL_TEXT_SIZE)
        .font_family(INTER_FAMILY)
        .text_color(bind(move |runtime| {
            let stage = Stage::of(step, current.get(runtime));
            theme.with(runtime, |theme| stage.label_color(theme))
        }))
        .text_center()
        .truncate()
        .w_full()
        .mt(LABEL_GAP)
}

pub(super) fn step_column(
    theme: Signal<Theme>,
    step: usize,
    current: Memo<usize>,
    label: String,
) -> Element<StackKind<(Dot, Element<TextKind>)>> {
    vstack((
        dot(theme, step, current),
        caption(theme, step, current, label),
    ))
    .flex_1()
    .min_w(0.0)
    .items_center()
}
