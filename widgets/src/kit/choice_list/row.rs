use geometry::{Color, CornerRadii, Insets, Point, Rect};
use reactive::{Memo, Signal};
use semantics::Role;
use view::{Cx, Element, IntoProp, Listeners, NodeId, Semantic, Styled, View, bind};

use super::{
    choice::Choice,
    metrics::{
        BORDER_WIDTH, DIMMED_ALPHA, LINE_GAP, MARK_DIAMETER, ROW_HEIGHT, ROW_PADDING, ROW_RADIUS,
        SELECTED_TINT, SUBTITLE_SIZE, TILE_GAP, TILE_ICON_SIZE, TILE_ICON_STROKE, TILE_RADIUS,
        TILE_SELECTED_ALPHA, TILE_SIZE, TITLE_SIZE,
    },
};
use crate::{
    canvas::canvas,
    kit::{GradientStops, Theme, icon::tinted_icon, radio_mark::RadioMarkKind},
    stack::{hstack, vstack},
    text::text,
    util::{color::scale_alpha, spring, themed::themed},
};

#[derive(Clone, Copy)]
pub(super) struct ListState {
    pub(super) theme: Signal<Theme>,
    pub(super) selected: Signal<Option<usize>>,
    pub(super) shown: Memo<Option<usize>>,
    pub(super) disabled: Signal<bool>,
}

fn faded(color: Color, dimmed: bool) -> Color {
    if dimmed {
        scale_alpha(color, DIMMED_ALPHA)
    } else {
        color
    }
}

fn overlay() -> Element<crate::stack::StackKind<()>> {
    vstack(()).absolute().inset(0.0).rounded(ROW_RADIUS)
}

fn border_layer(
    theme: Signal<Theme>,
    dimmed: Memo<bool>,
    pick: fn(&Theme) -> GradientStops,
    opacity: impl IntoProp<f32>,
) -> impl View {
    canvas(move |runtime, bounds, list| {
        let is_dimmed = dimmed.get(runtime);
        let stops = theme.with(runtime, |theme| {
            pick(theme).map(|color| faded(color, is_dimmed))
        });
        let half = BORDER_WIDTH / 2.0;
        list.stroke(
            Rect::new(Point::ZERO, bounds).inset(Insets::uniform(half)),
            CornerRadii::uniform(ROW_RADIUS - half),
            BORDER_WIDTH,
            stops.vertical(0.0, bounds.height),
        );
    })
    .absolute()
    .inset(0.0)
    .opacity(opacity)
}

fn glyph(
    body: &'static str,
    color: impl FnMut(&mut reactive::Runtime) -> Color + 'static,
    opacity: impl IntoProp<f32>,
) -> impl View {
    hstack(tinted_icon(body, TILE_ICON_STROKE, bind(color)).size(TILE_ICON_SIZE))
        .absolute()
        .inset(0.0)
        .items_center()
        .justify_center()
        .opacity(opacity)
}

fn tile(
    theme: Signal<Theme>,
    dimmed: Memo<bool>,
    selected: Memo<bool>,
    body: &'static str,
) -> impl View {
    let resting = vstack(())
        .absolute()
        .inset(0.0)
        .rounded(TILE_RADIUS)
        .bg(bind(move |runtime| {
            let is_dimmed = dimmed.get(runtime);
            theme.with(runtime, |theme| faded(theme.surface.sunken, is_dimmed))
        }))
        .opacity(bind(
            move |runtime| if selected.get(runtime) { 0.0 } else { 1.0 },
        ));
    let picked = vstack(())
        .absolute()
        .inset(0.0)
        .rounded(TILE_RADIUS)
        .bg(bind(move |runtime| {
            let is_dimmed = dimmed.get(runtime);
            theme.with(runtime, |theme| {
                faded(
                    scale_alpha(theme.accent.end, TILE_SELECTED_ALPHA),
                    is_dimmed,
                )
            })
        }))
        .opacity(bind(
            move |runtime| if selected.get(runtime) { 1.0 } else { 0.0 },
        ));
    let muted_glyph = glyph(
        body,
        move |runtime| {
            let is_dimmed = dimmed.get(runtime);
            theme.with(runtime, |theme| faded(theme.text.icon_muted, is_dimmed))
        },
        1.0,
    );
    let accent_glyph = glyph(
        body,
        move |runtime| {
            let is_dimmed = dimmed.get(runtime);
            theme.with(runtime, |theme| faded(theme.accent.end, is_dimmed))
        },
        bind(move |runtime| if selected.get(runtime) { 1.0 } else { 0.0 }),
    );
    vstack((resting, picked, muted_glyph, accent_glyph))
        .relative()
        .size(TILE_SIZE)
        .shrink(0.0)
        .rounded(TILE_RADIUS)
}

pub(super) fn build_row(cx: &mut Cx, list: ListState, index: usize, choice: &Choice) -> NodeId {
    let ListState {
        theme,
        selected,
        shown,
        disabled,
    } = list;
    let available = choice.available;
    let is_selected = cx.memo(move |runtime| shown.get(runtime) == Some(index));
    let dimmed = cx.memo(move |runtime| disabled.get(runtime) || !available);
    let hovered = cx.signal(false);

    let hover_wash = overlay()
        .bg(themed(theme, |theme| theme.surface.hover))
        .opacity(bind(move |runtime| {
            if hovered.get(runtime) && !dimmed.get(runtime) {
                1.0
            } else {
                0.0
            }
        }))
        .transition(spring::snappy());
    let selection_tint = overlay()
        .bg(themed(theme, |theme| theme.accent.end))
        .opacity(bind(move |runtime| {
            if is_selected.get(runtime) {
                SELECTED_TINT
            } else {
                0.0
            }
        }));
    let hairline = border_layer(
        theme,
        dimmed,
        |theme| GradientStops::flat(theme.surface.border),
        1.0,
    );
    let accent_border = border_layer(
        theme,
        dimmed,
        |theme| theme.accent,
        bind(move |runtime| if is_selected.get(runtime) { 1.0 } else { 0.0 }),
    );
    let leading = choice
        .icon
        .map(|body| tile(theme, dimmed, is_selected, body).boxed());
    let has_tile = leading.is_some();

    let title = text(choice.title.clone())
        .text_size(TITLE_SIZE)
        .truncate()
        .text_color(bind(move |runtime| {
            let is_dimmed = dimmed.get(runtime);
            theme.with(runtime, |theme| faded(theme.text.primary, is_dimmed))
        }));
    let subtitle = (!choice.subtitle.is_empty()).then(|| {
        text(choice.subtitle.clone())
            .text_size(SUBTITLE_SIZE)
            .truncate()
            .text_color(bind(move |runtime| {
                let is_dimmed = dimmed.get(runtime);
                theme.with(runtime, |theme| faded(theme.text.muted, is_dimmed))
            }))
    });
    let lines = vstack((title, subtitle))
        .flex_1()
        .min_w(0.0)
        .gap(LINE_GAP)
        .justify_center()
        .ml(if has_tile { TILE_GAP } else { 0.0 })
        .mr(ROW_PADDING);
    let mark = Element::new(RadioMarkKind {
        diameter: MARK_DIAMETER,
        selected: is_selected,
        dimmed,
    })
    .hidden(dimmed);

    let row = hstack((
        hover_wash,
        selection_tint,
        hairline,
        accent_border,
        leading,
        lines,
        mark,
    ))
    .role(Role::RadioButton)
    .focusable(false)
    .label(choice.title.clone())
    .description(choice.subtitle.clone())
    .disabled(!available)
    .relative()
    .items_center()
    .h(ROW_HEIGHT)
    .px(ROW_PADDING)
    .shrink(0.0)
    .rounded(ROW_RADIUS)
    .bg(bind(move |runtime| {
        let is_dimmed = dimmed.get(runtime);
        theme.with(runtime, |theme| faded(theme.surface.raised, is_dimmed))
    }))
    .on_pointer_enter(move |cx, _| hovered.set(cx, true))
    .on_pointer_leave(move |cx, _| hovered.set(cx, false))
    .on_click(move |cx, _| {
        if !dimmed.get(cx) && selected.get(cx) != Some(index) {
            selected.set(cx, Some(index));
        }
    });
    let row = cx.build(row);
    cx.bind_checked(row, move |runtime| is_selected.get(runtime));
    row
}
