use geometry::{Size, Vector};
use reactive::{Runtime, Signal};
use view::{Cx, NodeId, Styled, bind};

use super::{
    editor::{Editor, Segment},
    metrics::{CARET_WIDTH, SELECTION_ALPHA, UNFOCUSED_SELECTION_ALPHA},
};
use crate::{
    kit::Theme,
    stack::{hstack, vstack},
    text::text,
    util::{color::scale_alpha, themed::themed},
};

pub(super) struct Content {
    pub(super) viewport: NodeId,
    pub(super) row_size: Signal<Size>,
    pub(super) scroll: Signal<f32>,
}

pub(super) struct ContentInput {
    pub(super) theme: Signal<Theme>,
    pub(super) editor: Editor,
    pub(super) focused: Signal<bool>,
    pub(super) disabled: Signal<bool>,
    pub(super) placeholder: String,
    pub(super) inset_left: f32,
    pub(super) inset_right: f32,
}

fn caret(
    cx: &mut Cx,
    theme: Signal<Theme>,
    hidden: impl FnMut(&mut Runtime) -> bool + 'static,
) -> NodeId {
    let bar = vstack(())
        .absolute()
        .top(0.0)
        .bottom(0.0)
        .left(0.0)
        .w(CARET_WIDTH)
        .bg(themed(theme, |theme| theme.accent.end))
        .hidden(bind(hidden));
    cx.build(vstack(bar).w(0.0).shrink(0.0).relative().self_stretch())
}

pub(super) fn mount_content(cx: &mut Cx, input: ContentInput) -> Content {
    let ContentInput {
        theme,
        editor,
        focused,
        disabled,
        placeholder,
        inset_left,
        inset_right,
    } = input;
    let segment = move |segment| bind(move |runtime| editor.segment(runtime, segment));
    let selection_color = bind(move |runtime| {
        let alpha = if focused.get(runtime) {
            SELECTION_ALPHA
        } else {
            UNFOCUSED_SELECTION_ALPHA
        };
        theme.with(runtime, |theme| scale_alpha(theme.accent.end, alpha))
    });
    let before = cx.build(text(segment(Segment::Before)).shrink(0.0));
    let selected = cx.build(
        text(segment(Segment::Selected))
            .bg(selection_color)
            .shrink(0.0),
    );
    let after = cx.build(text(segment(Segment::After)).shrink(0.0));
    let caret_at = move |at_end: bool| {
        move |runtime: &mut Runtime| {
            let selection = editor.selection_now(runtime);
            let shown = focused.get(runtime) && !disabled.get(runtime);
            !(shown && (selection.caret > selection.anchor) == at_end)
        }
    };
    let caret_before = caret(cx, theme, caret_at(false));
    let caret_after = caret(cx, theme, caret_at(true));

    let scroll = cx.signal(0.0_f32);
    let row = cx.build(
        hstack(())
            .items_center()
            .shrink(0.0)
            .translate(bind(move |runtime| Vector::new(-scroll.get(runtime), 0.0))),
    );
    cx.adopt_children(
        row,
        vec![before, caret_before, selected, caret_after, after],
    );
    let placeholder = hstack(
        text(placeholder)
            .shrink(0.0)
            .text_color(themed(theme, |theme| theme.popover.muted_text)),
    )
    .absolute()
    .inset(0.0)
    .items_center()
    .hidden(bind(move |runtime| {
        editor.value.with(runtime, |value| !value.is_empty())
    }));
    let viewport = cx.build(
        hstack(())
            .absolute()
            .top(0.0)
            .bottom(0.0)
            .left(inset_left)
            .right(inset_right)
            .items_center()
            .overflow_hidden(),
    );
    let placeholder = cx.build(placeholder);
    cx.adopt_children(viewport, vec![placeholder, row]);

    let (viewport_size, row_size) = (cx.observe_size(viewport), cx.observe_size(row));
    let (before_size, selected_size) = (cx.observe_size(before), cx.observe_size(selected));
    cx.effect(move |runtime| {
        let viewport_width = viewport_size.get(runtime).width;
        let content_width = row_size.get(runtime).width;
        let selection = editor.selection_now(runtime);
        let selected_width = if selection.caret > selection.anchor {
            selected_size.get(runtime).width
        } else {
            0.0
        };
        let caret_x = before_size.get(runtime).width + selected_width;
        let current = scroll.with_untracked(runtime, |offset| *offset);
        let following = current
            .max(caret_x + CARET_WIDTH - viewport_width)
            .min(caret_x)
            .min(content_width - viewport_width)
            .max(0.0);
        if following != current {
            scroll.set(runtime, following);
        }
    });
    Content {
        viewport,
        row_size,
        scroll,
    }
}
