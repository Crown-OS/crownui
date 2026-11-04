use reactive::{Signal, Store};
use semantics::Role;
use view::{Element, Listeners, Semantic, Styled, bind};

use super::{
    handlers::{SegmentHandlers, run},
    segment::PathSegment,
};
use crate::{
    kit::{Theme, icon::tinted_icon, icons},
    stack::{StackKind, hstack},
    text::text,
    util::{spring, themed::themed},
};

pub(super) const ENTRY_GAP: f32 = 2.0;
pub(super) const ELLIPSIS_WIDTH: f32 = 2.0 * CRUMB_H_PADDING + ICON_SIZE;
const FOLDED_LABEL: &str = "More";
const CRUMB_H_PADDING: f32 = 6.0;
const CRUMB_V_PADDING: f32 = 4.0;
const CRUMB_RADIUS: f32 = 8.0;
const CRUMB_GAP: f32 = 6.0;
const ICON_SIZE: f32 = 16.0;
const ICON_STROKE: f32 = 1.8;
const CHEVRON_SIZE: f32 = 14.0;

/// Where a crumb leads: a fixed segment, or the deepest folded one behind the ellipsis.
#[derive(Clone, Copy)]
pub(super) enum Target {
    Segment(usize),
    Folded(reactive::Memo<usize>),
}

impl Target {
    fn index(self, runtime: &mut reactive::Runtime) -> usize {
        match self {
            Self::Segment(index) => index,
            Self::Folded(folded) => folded.get(runtime).saturating_sub(1),
        }
    }
}

/// What a crumb shows and how it behaves.
pub(super) struct Crumb<'a> {
    pub(super) theme: Signal<Theme>,
    pub(super) segment: &'a PathSegment,
    pub(super) target: Target,
    pub(super) current: bool,
    pub(super) handlers: Store<SegmentHandlers>,
    pub(super) drop_target: Signal<Option<usize>>,
}

/// One hoverable pill of the path, a drop target for its folder.
pub(super) fn crumb(crumb: Crumb<'_>) -> Element<StackKind<impl view::ViewSeq>> {
    let Crumb {
        theme,
        segment,
        target,
        current,
        handlers,
        drop_target,
    } = crumb;
    let glyph = segment.icon.map(|body| {
        tinted_icon(
            body,
            ICON_STROKE,
            themed(theme, move |theme| label_color(theme, current)),
        )
        .size(ICON_SIZE)
        .shrink(0.0)
    });
    let name = (!segment.label.is_empty()).then(|| {
        let name = text(segment.label.clone()).truncate();
        if current { name.font_semibold() } else { name }
    });
    let lit = move |runtime: &mut reactive::Runtime| {
        let index = target.index(runtime);
        drop_target.get(runtime) == Some(index)
    };
    hstack((glyph, name))
        .role(Role::Button)
        .label(if segment.label.is_empty() {
            FOLDED_LABEL.to_owned()
        } else {
            segment.label.clone()
        })
        .items_center()
        .shrink(0.0)
        .gap(CRUMB_GAP)
        .px(CRUMB_H_PADDING)
        .py(CRUMB_V_PADDING)
        .rounded(CRUMB_RADIUS)
        .text_color(themed(theme, move |theme| label_color(theme, current)))
        .bg(bind(move |runtime| {
            let wash = theme.with(runtime, |theme| theme.surface.hover);
            if lit(runtime) {
                wash
            } else {
                geometry::TRANSPARENT
            }
        }))
        .hover(|style| style.bg(themed(theme, |theme| theme.surface.hover)))
        .transition(spring::crisp())
        .on_click(move |cx, _| {
            let index = target.index(cx);
            run(
                cx,
                handlers,
                |all| &mut all.select,
                |cx, select| select(cx, index),
            );
        })
        .on_drag_over(move |cx, event| {
            let index = target.index(cx);
            run(
                cx,
                handlers,
                |all| &mut all.drag_over,
                |cx, over| over(cx, index, event),
            );
            let lit = event.is_accepted().then_some(index);
            if drop_target.get(cx) != lit {
                drop_target.set(cx, lit);
            }
        })
        .on_drag_leave(move |cx, _| {
            if drop_target.get(cx).is_some() {
                drop_target.set(cx, None);
            }
        })
        .on_drop(move |cx, event| {
            let index = target.index(cx);
            drop_target.set(cx, None);
            run(
                cx,
                handlers,
                |all| &mut all.drop,
                |cx, drop| drop(cx, index, event),
            );
        })
}

/// The muted chevron between two crumbs.
pub(super) fn chevron(theme: Signal<Theme>) -> Element<crate::svg::SvgKind> {
    tinted_icon(
        icons::CHEVRON_RIGHT,
        ICON_STROKE,
        themed(theme, |theme| theme.text.disabled),
    )
    .size(CHEVRON_SIZE)
    .shrink(0.0)
    .role(Role::Group)
}

fn label_color(theme: &Theme, current: bool) -> geometry::Color {
    if current {
        theme.text.primary
    } else {
        theme.text.muted
    }
}
