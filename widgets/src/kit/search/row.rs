use layouting::{Align, Dimension, FlexDirection, Position};
use view::{Cx, Element, Kind, Listener, NodeId, SizeProperty, StyleProp, Styled};

use super::{
    metrics::{
        DETAIL_GAP, DETAIL_TEXT_SIZE, ICON_STROKE, ROW_H_PADDING, ROW_HEIGHT, ROW_ICON,
        ROW_ICON_GAP,
    },
    picker::Picker,
    state::SearchState,
    suggestion::Suggestion,
};
use crate::{
    kit::{Theme, icon::tinted_icon},
    stack::hstack,
    text::text,
    util::themed::themed,
};
use reactive::Signal;
use semantics::Role;

pub(super) struct SuggestionRowKind {
    pub(super) theme: Signal<Theme>,
    pub(super) state: SearchState,
    pub(super) picker: Picker,
    pub(super) index: usize,
    pub(super) suggestion: Suggestion,
}

impl Kind for SuggestionRowKind {
    const NAME: &'static str = "kit_search_row";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let Self {
            theme,
            state,
            picker,
            index,
            suggestion,
        } = self;
        cx.set_role(node, Role::ListItem);
        for prop in [
            StyleProp::FlexDirection(FlexDirection::Row),
            StyleProp::AlignItems(Align::Center),
            StyleProp::Position(Position::Relative),
            StyleProp::Size(SizeProperty::Height, Dimension::Length(ROW_HEIGHT)),
            StyleProp::Padding(view::Edges::Horizontal, ROW_H_PADDING),
            StyleProp::Shrink(0.0),
        ] {
            cx.set_default_style(node, prop);
        }

        let mut parts = Vec::with_capacity(3);
        let icon_width = suggestion.icon.map_or(0.0, |body| {
            let glyph = tinted_icon(
                body,
                ICON_STROKE,
                themed(theme, |theme| theme.popover.muted_text),
            )
            .size(ROW_ICON)
            .mr(ROW_ICON_GAP)
            .shrink(0.0);
            parts.push(cx.build(glyph));
            ROW_ICON + ROW_ICON_GAP
        });
        let label = cx.build(
            text(suggestion.label)
                .shrink(0.0)
                .text_color(themed(theme, |theme| theme.popover.text)),
        );
        let label_column = cx.build(
            hstack(())
                .items_center()
                .flex_1()
                .min_w(0.0)
                .overflow_hidden(),
        );
        cx.adopt_children(label_column, vec![label]);
        parts.push(label_column);

        if let Some(detail) = suggestion.detail {
            let detail = cx.build(
                hstack(
                    text(detail)
                        .shrink(0.0)
                        .text_size(DETAIL_TEXT_SIZE)
                        .text_color(themed(theme, |theme| theme.popover.muted_text)),
                )
                .absolute()
                .right(ROW_H_PADDING)
                .top(0.0)
                .bottom(0.0)
                .items_center(),
            );
            let (row_size, label_size, detail_size) = (
                cx.observe_size(node),
                cx.observe_size(label),
                cx.observe_size(detail),
            );
            cx.bind_style(detail, move |runtime| {
                let available = row_size.get(runtime).width - 2.0 * ROW_H_PADDING - icon_width;
                let wanted =
                    label_size.get(runtime).width + DETAIL_GAP + detail_size.get(runtime).width;
                StyleProp::Hidden(wanted > available)
            });
            parts.push(detail);
        }
        cx.adopt_children(node, parts);

        let highlight = state.highlight;
        cx.add_listener(
            node,
            Listener::PointerDown(Box::new(|_, event| event.prevent_default())),
        );
        cx.add_listener(
            node,
            Listener::PointerMove(Box::new(move |cx, _| {
                if highlight.get(cx) != Some(index) {
                    highlight.set(cx, Some(index));
                }
            })),
        );
        cx.add_listener(
            node,
            Listener::Click(Box::new(move |cx, _| picker.pick(cx, index))),
        );
    }
}

pub(super) fn suggestion_row(
    theme: Signal<Theme>,
    state: SearchState,
    picker: Picker,
    index: usize,
    suggestion: Suggestion,
) -> Element<SuggestionRowKind> {
    Element::new(SuggestionRowKind {
        theme,
        state,
        picker,
        index,
        suggestion,
    })
}
