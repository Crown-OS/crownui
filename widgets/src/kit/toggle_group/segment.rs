use geometry::Color;
use reactive::{Runtime, Signal};
use semantics::Role;
use view::{AnyView, Element, Listeners, Semantic, Styled, View, bind};

use crate::stack::StackKind;

use super::{
    item::ToggleItem,
    metrics::{HOVER_WASH, ICON_SIZE, ICON_STROKE, LABEL_SIZE, SEGMENT_HEIGHT, SEGMENT_PADDING},
    selection::Selection,
};
use crate::{
    kit::{Theme, icon::tinted_icon},
    stack::hstack,
    text::text,
    util::{color::scale_alpha, spring},
};

#[derive(PartialEq, Eq)]
enum Announce {
    Yes,
    No,
}

fn face(
    item: &ToggleItem,
    announce: Announce,
    color: impl FnMut(&mut Runtime) -> Color + 'static,
) -> AnyView {
    let silent = announce == Announce::No;
    match item.icon {
        Some(body) => tinted_icon(body, ICON_STROKE, bind(color))
            .size(ICON_SIZE)
            .role(if silent { Role::Group } else { Role::Image })
            .boxed(),
        None => text(item.label.clone())
            .text_size(LABEL_SIZE)
            .text_color(bind(color))
            .role(if silent { Role::Group } else { Role::Text })
            .boxed(),
    }
}

#[derive(Clone, Copy)]
pub(super) struct SegmentState {
    pub(super) theme: Signal<Theme>,
    pub(super) disabled: Signal<bool>,
    pub(super) selection: Selection,
    pub(super) hovered: Signal<Option<usize>>,
}

pub(super) fn segment(
    state: &SegmentState,
    index: usize,
    item: &ToggleItem,
) -> Element<StackKind<(AnyView, impl view::View)>> {
    let SegmentState {
        theme,
        disabled,
        selection,
        hovered,
    } = *state;
    let resting = face(item, Announce::Yes, move |runtime| {
        let is_disabled = disabled.get(runtime);
        theme.with(runtime, |theme| {
            if is_disabled {
                theme.text.disabled
            } else {
                theme.text.muted
            }
        })
    });
    let emphasized = hstack(face(item, Announce::No, move |runtime| {
        theme.with(runtime, |theme| theme.text.primary)
    }))
    .absolute()
    .inset(0.0)
    .items_center()
    .justify_center()
    .opacity(bind(move |runtime| {
        let chosen = selection.get(runtime) == index && !disabled.get(runtime);
        if chosen { 1.0 } else { 0.0 }
    }))
    .transition(spring::snappy());
    hstack((resting, emphasized))
        .role(Role::RadioButton)
        .focusable(false)
        .label(item.label.clone())
        .relative()
        .items_center()
        .justify_center()
        .h(SEGMENT_HEIGHT)
        .px(SEGMENT_PADDING)
        .rounded(SEGMENT_HEIGHT / 2.0)
        .bg(bind(move |runtime| {
            let washed = hovered.get(runtime) == Some(index)
                && selection.get(runtime) != index
                && !disabled.get(runtime);
            if washed {
                theme.with(runtime, |theme| {
                    scale_alpha(theme.surface.hover, HOVER_WASH)
                })
            } else {
                geometry::TRANSPARENT
            }
        }))
        .on_click(move |cx, _| {
            if !disabled.get(cx) {
                selection.choose(cx, index);
            }
        })
        .on_pointer_enter(move |cx, _| hovered.set(cx, Some(index)))
        .on_pointer_leave(move |cx, _| {
            if hovered.get(cx) == Some(index) {
                hovered.set(cx, None);
            }
        })
}
