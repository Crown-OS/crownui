use geometry::Vector;
use reactive::Signal;
use view::{Listeners, Modifier, Semantic, Styled, VariantStyle, View, bind, for_each};

use super::{
    metrics::{APPEAR_SCALE, APPEAR_SLIDE, PANEL_PADDING, PANEL_RADIUS, ROW_HEIGHT, ROW_RADIUS},
    picker::Picker,
    row::suggestion_row,
    state::SearchState,
};
use crate::{
    kit::Theme,
    stack::vstack,
    util::{
        glass::{glass_plate, glass_shadow},
        spring,
        themed::themed,
    },
};

fn hidden_look(style: VariantStyle) -> VariantStyle {
    style
        .opacity(0.0)
        .scale(APPEAR_SCALE)
        .translate_y(-APPEAR_SLIDE)
}

fn highlight_pill(theme: Signal<Theme>, state: SearchState) -> impl View {
    let mut last_row = 0;
    vstack(())
        .absolute()
        .top(PANEL_PADDING)
        .left(PANEL_PADDING)
        .right(PANEL_PADDING)
        .h(ROW_HEIGHT)
        .rounded(ROW_RADIUS)
        .border(1.0)
        .bg(themed(theme, |theme| theme.menu.selected_bg))
        .border_color(themed(theme, |theme| theme.menu.selected_border))
        .opacity(bind(move |runtime| {
            if state.highlight.get(runtime).is_some() {
                1.0
            } else {
                0.0
            }
        }))
        .translate(bind(move |runtime| {
            if let Some(row) = state.highlight.get(runtime) {
                last_row = row;
            }
            Vector::new(0.0, last_row as f32 * ROW_HEIGHT)
        }))
        .transition(spring::snappy())
}

pub(super) fn panel(theme: Signal<Theme>, state: SearchState, picker: Picker) -> impl View {
    let rows = for_each(
        state.rows,
        |item| item.clone(),
        move |cx, item| {
            let (index, suggestion) = item.with_untracked(cx, Clone::clone);
            let label = suggestion.label.clone();
            suggestion_row(theme, state, picker, index, suggestion)
                .label(label)
                .focusable(false)
        },
    )
    .flex_col();
    vstack((
        glass_plate(theme, PANEL_RADIUS),
        highlight_pill(theme, state),
        rows,
    ))
    .relative()
    .rounded(PANEL_RADIUS)
    .modifier(Modifier::Bound(Box::new(glass_shadow(theme))))
    .p(PANEL_PADDING)
    .origin(0.5, 0.0)
    .enter(hidden_look)
    .exit(hidden_look)
    .transition(spring::crisp())
    .on_pointer_down(|_, event| event.prevent_default())
}
