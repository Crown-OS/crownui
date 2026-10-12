use std::time::Duration;

use motion::{CubicBezier, Curve, TimingCurve};
use reactive::{Runtime, Signal};
use semantics::Role;
use view::{Cx, Element, Semantic, Styled, View, bind, for_each};

use super::{handles::SelectHandles, metrics::*};
use crate::{
    kit::Theme,
    stack::{StackKind, vstack},
    text::text,
    util::spring,
};

#[derive(Debug, Clone, PartialEq)]
struct ShownValue {
    text: String,
    placeholder: bool,
}

fn shown_value(runtime: &mut Runtime, handles: SelectHandles) -> ShownValue {
    let selected = handles.selected.get(runtime);
    let option = handles.options.with(runtime, |options| {
        selected.and_then(|index| options.get(index).cloned())
    });
    match option {
        Some(text) => ShownValue {
            text,
            placeholder: false,
        },
        None => ShownValue {
            text: handles.placeholder.get(runtime),
            placeholder: true,
        },
    }
}

/// The incoming value waits for the outgoing one to reach the bottom of its dip.
fn delayed_rise() -> Curve {
    Curve::Timing(TimingCurve {
        duration: Duration::from_millis(VALUE_FADE_MILLIS),
        easing: CubicBezier::new(0.7, 0.0, 0.9, 1.0),
        repeats: false,
    })
}

fn value_text(
    theme: Signal<Theme>,
    disabled: Signal<bool>,
    value: ShownValue,
    fades_in: bool,
) -> Element<StackKind<Element<crate::text::TextKind>>> {
    let muted = value.placeholder;
    let label = text(value.text).truncate().text_color(bind(move |runtime| {
        let dimmed = muted || disabled.get(runtime);
        theme.with(runtime, |theme| {
            if dimmed {
                theme.popover.muted_text
            } else {
                theme.popover.text
            }
        })
    }));
    let label = if fades_in {
        label
            .enter(|style| style.opacity(0.0))
            .transition(delayed_rise())
    } else {
        label
    };
    vstack(label).exit(|style| style.opacity(0.0))
}

/// The selected option, or the placeholder, replaced by a dip: the old text fades out and the new
/// text fades in once it is gone. Whatever the select is built with is simply there.
pub(super) fn value_label(
    cx: &mut Cx,
    theme: Signal<Theme>,
    disabled: Signal<bool>,
    handles: SelectHandles,
) -> impl View {
    let initial = shown_value(cx, handles);
    let shown = cx.signal(vec![initial]);
    cx.effect(move |runtime| {
        let next = shown_value(runtime, handles);
        shown.set(runtime, vec![next]);
    });
    let mut built_once = false;
    for_each(
        shown,
        |value| (value.placeholder, value.text.clone()),
        move |cx, item| {
            let fades_in = std::mem::replace(&mut built_once, true);
            let value = item.with_untracked(cx, Clone::clone);
            value_text(theme, disabled, value, fades_in)
        },
    )
    .role(Role::Group)
    .w(VALUE_TEXT_WIDTH)
    .ml(TRIGGER_H_PADDING)
    .shrink(0.0)
    .transition(spring::crisp())
}
