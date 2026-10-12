use geometry::Color;
use reactive::Runtime;
use semantics::Role;
use view::{Cx, Element, Kind, Listener, NodeId, Semantic, StyleProp, Styled, bind};

use super::{entry::RowSpec, handles::MenuHandles, metrics::*, tint::row_content_color};
use crate::{
    kit::{TEXT_SIZE, icon::tinted_icon, icons, use_theme},
    stack::vstack,
    text::text,
    util::anchored_popup::instant,
};

/// The kind behind a popup menu's pickable rows.
pub struct PopupMenuRowKind {
    spec: RowSpec,
    index: usize,
    handles: MenuHandles,
}

impl Kind for PopupMenuRowKind {
    const NAME: &'static str = "kit_popup_menu_row";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let disabled = cx.observe_disabled(node);
        let Self {
            spec,
            index,
            handles,
        } = self;
        let highlighted = handles.highlighted;
        let is_highlighted = move |runtime: &mut Runtime| highlighted.get(runtime) == Some(index);
        let destructive = spec.destructive;
        let content = move |runtime: &mut Runtime| -> Color {
            let (lit, dimmed) = (is_highlighted(runtime), disabled.get(runtime));
            theme.with(runtime, |theme| {
                row_content_color(theme, lit, dimmed, destructive)
            })
        };

        cx.set_role(node, Role::MenuItem);
        cx.bind_style(node, move |runtime| StyleProp::TextColor(content(runtime)));
        cx.bind_style(node, move |runtime| {
            let lit = is_highlighted(runtime);
            StyleProp::Background(lit.then(|| theme.with(runtime, |theme| theme.menu.selected_bg)))
        });
        if let Some(flag) = spec.checked {
            cx.bind_checked(node, move |runtime| flag.resolve(runtime));
        }
        cx.add_listener(
            node,
            Listener::PointerEnter(Box::new(move |cx, _| {
                if !disabled.get(cx) {
                    highlighted.set(cx, Some(index));
                }
            })),
        );
        cx.add_listener(
            node,
            Listener::PointerLeave(Box::new(move |cx, _| {
                if highlighted.get(cx) == Some(index) {
                    highlighted.set(cx, None);
                }
            })),
        );
        cx.add_listener(
            node,
            Listener::Click(Box::new(move |cx, event| {
                handles.pick(cx, index);
                event.stop_propagation();
            })),
        );

        let check = spec.checked.map(|flag| {
            tinted_icon(icons::CHECK, ICON_STROKE, bind(content))
                .size(CHECK_SIZE)
                .hidden(bind(move |runtime| !flag.resolve(runtime)))
        });
        let check_column = vstack(check).w(CHECK_COLUMN).shrink(0.0);
        let label = text(spec.label).truncate().shrink(1.0).min_w(0.0);
        cx.set_children(node, (check_column, label));
    }
}

pub(super) fn pickable_row(
    spec: RowSpec,
    index: usize,
    handles: MenuHandles,
) -> Element<PopupMenuRowKind> {
    let (label, disabled) = (spec.label.clone(), spec.disabled);
    Element::new(PopupMenuRowKind {
        spec,
        index,
        handles,
    })
    .label(label)
    .disabled(disabled)
    .focusable(false)
    .flex_row()
    .items_center()
    .self_stretch()
    .h(ROW_HEIGHT)
    .px(ROW_H_PADDING)
    .rounded(ROW_RADIUS)
    .shrink(0.0)
    .text_size(TEXT_SIZE)
    .transition(instant())
}
