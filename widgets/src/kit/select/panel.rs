use geometry::Vector;
use semantics::Role;
use view::{Cx, Element, Kind, Listener, NodeId, StyleProp, Styled, bind, show};

use super::{handles::SelectHandles, metrics::*, row::option_row};
use crate::{
    kit::{INTER_FAMILY, TEXT_SIZE, icon::tinted_icon, icons, use_theme},
    stack::vstack,
    util::{
        spring,
        themed::{bind_themed_style, themed},
    },
};

const PILL_WIDTH: f32 = TRIGGER_WIDTH - PANEL_PADDING * 2.0;
const CHECK_X: f32 = TRIGGER_WIDTH - PANEL_PADDING - ROW_H_PADDING - CHECK_BOX;
const CHECK_Y: f32 = (ROW_HEIGHT - CHECK_BOX) / 2.0;

fn highlight_pill(
    theme: reactive::Signal<crate::kit::Theme>,
    handles: SelectHandles,
) -> Element<crate::stack::StackKind<()>> {
    let mut resting_row = 0;
    vstack(())
        .absolute()
        .left(0.0)
        .top(0.0)
        .w(PILL_WIDTH)
        .h(ROW_HEIGHT)
        .rounded(ROW_RADIUS)
        .bg(themed(theme, |theme| theme.popover.hover_bg))
        .translate(bind(move |runtime| {
            if let Some(row) = handles.highlighted.get(runtime) {
                resting_row = row;
            }
            Vector::new(
                PANEL_PADDING,
                PANEL_PADDING + resting_row as f32 * ROW_HEIGHT,
            )
        }))
        .transition(spring::snappy())
        .enter(|style| style.opacity(0.0))
        .exit(|style| style.opacity(0.0))
}

fn checkmark(
    theme: reactive::Signal<crate::kit::Theme>,
    handles: SelectHandles,
) -> Element<crate::svg::SvgKind> {
    let row = move |runtime: &mut reactive::Runtime| {
        let count = handles.option_count(runtime);
        handles.selected.get(runtime).filter(|&row| row < count)
    };
    tinted_icon(
        icons::CHECK,
        ICON_STROKE,
        themed(theme, |theme| theme.popover.text),
    )
    .size(CHECK_BOX)
    .absolute()
    .left(0.0)
    .top(0.0)
    .hidden(bind(move |runtime| row(runtime).is_none()))
    .translate(bind(move |runtime| {
        let row = row(runtime).unwrap_or(0);
        Vector::new(CHECK_X, PANEL_PADDING + row as f32 * ROW_HEIGHT + CHECK_Y)
    }))
    .transition(spring::snappy())
}

/// The kind behind the floating panel of a select.
pub struct SelectPanelKind {
    handles: SelectHandles,
}

impl Kind for SelectPanelKind {
    const NAME: &'static str = "kit_select_panel";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let handles = self.handles;
        cx.set_role(node, Role::List);
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::Background(Some(theme.popover.bg))
        });
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::BorderColor(theme.popover.border)
        });
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::TextColor(theme.popover.text)
        });
        cx.add_listener(
            node,
            Listener::PointerDown(Box::new(|_, event| event.prevent_default())),
        );
        cx.add_listener(
            node,
            Listener::Click(Box::new(|_, event| event.stop_propagation())),
        );
        cx.add_listener(
            node,
            Listener::PointerLeave(Box::new(move |cx, _| handles.highlighted.set(cx, None))),
        );

        let pill = show(
            bind(move |runtime| handles.highlighted.get(runtime).is_some()),
            move || highlight_pill(theme, handles),
            || vstack(()),
        )
        .absolute()
        .left(0.0)
        .top(0.0);
        let options = handles.options.with_untracked(cx, Clone::clone);
        let rows: Vec<NodeId> = options
            .into_iter()
            .enumerate()
            .map(|(index, label)| cx.build(option_row(label, index, handles)))
            .collect();
        let mut children = vec![cx.build(pill)];
        children.extend(rows);
        children.push(cx.build(checkmark(theme, handles)));
        cx.adopt_children(node, children);
    }
}

pub(super) fn select_panel(handles: SelectHandles) -> Element<SelectPanelKind> {
    Element::new(SelectPanelKind { handles })
        .relative()
        .flex_col()
        .items_stretch()
        .w(TRIGGER_WIDTH)
        .p(PANEL_PADDING)
        .rounded(PANEL_RADIUS)
        .border(BORDER_WIDTH)
        .text_size(TEXT_SIZE)
        .font_family(INTER_FAMILY)
}
