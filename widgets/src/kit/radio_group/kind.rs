use input::{Key, NamedKey};
use layouting::Align;
use reactive::Signal;
use semantics::Role;
use view::{Cx, Element, Kind, Listener, Listeners, NodeId, Semantic, StyleProp, Styled, bind};
use windowing::CursorIcon;

use super::{
    metrics::{CIRCLE_DIAMETER, LABEL_GAP, ROW_PITCH},
    navigation::{Step, step},
};
use crate::{
    kit::{
        INTER_FAMILY, TEXT_SIZE,
        radio_mark::{RadioMarkKind, muted},
        use_theme,
    },
    stack::hstack,
    text::text,
};

/// The kind behind [`radio_group`](super::radio_group).
pub struct RadioGroupKind {
    pub(super) options: Vec<String>,
    pub(super) selected: Signal<Option<usize>>,
}

fn step_for(key: NamedKey) -> Option<Step> {
    match key {
        NamedKey::ArrowUp | NamedKey::ArrowLeft => Some(Step::Previous),
        NamedKey::ArrowDown | NamedKey::ArrowRight => Some(Step::Next),
        _ => None,
    }
}

impl Kind for RadioGroupKind {
    const NAME: &'static str = "kit_radio_group";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let Self { options, selected } = self;
        let count = options.len();
        let shown = cx.memo(move |runtime| selected.get(runtime).filter(|index| *index < count));
        let disabled = cx.observe_disabled(node);
        let dimmed = cx.memo(move |runtime| disabled.get(runtime));

        cx.set_role(node, Role::Group);
        for prop in [
            StyleProp::AlignSelf(Align::Start),
            StyleProp::Shrink(0.0),
            StyleProp::Cursor(CursorIcon::Pointer),
            StyleProp::TextSize(TEXT_SIZE),
            StyleProp::FontFamily(INTER_FAMILY.to_owned()),
        ] {
            cx.set_default_style(node, prop);
        }
        let labels = options.clone();
        cx.bind_value(node, move |runtime| {
            shown
                .get(runtime)
                .and_then(|index| labels.get(index).cloned())
                .unwrap_or_default()
        });

        let mut rows = Vec::with_capacity(count);
        for (index, label) in options.into_iter().enumerate() {
            let is_selected = cx.memo(move |runtime| shown.get(runtime) == Some(index));
            let mark = Element::new(RadioMarkKind {
                diameter: CIRCLE_DIAMETER,
                selected: is_selected,
                dimmed,
            });
            let caption = text(label).ml(LABEL_GAP).text_color(bind(move |runtime| {
                let is_dimmed = dimmed.get(runtime);
                theme.with(runtime, |theme| muted(theme.radio.text, is_dimmed))
            }));
            let row = hstack((mark, caption))
                .role(Role::RadioButton)
                .focusable(false)
                .items_center()
                .h(ROW_PITCH)
                .on_click(move |cx, _| {
                    if !disabled.get(cx) && selected.get(cx) != Some(index) {
                        selected.set(cx, Some(index));
                    }
                });
            let row = cx.build(row);
            cx.bind_checked(row, move |runtime| is_selected.get(runtime));
            rows.push(row);
        }
        cx.adopt_children(node, rows);

        cx.add_listener(
            node,
            Listener::KeyDown(Box::new(move |cx, event| {
                let Key::Named(key) = event.key else {
                    return;
                };
                let Some(direction) = step_for(key) else {
                    return;
                };
                event.prevent_default();
                if disabled.get(cx) {
                    return;
                }
                let current = shown.get(cx);
                if let Some(next) = step(current, count, direction) {
                    selected.set(cx, Some(next));
                }
            })),
        );
    }
}
