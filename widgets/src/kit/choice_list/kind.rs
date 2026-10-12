use input::{Key, NamedKey};
use layouting::Align;
use reactive::{Memo, Signal};
use semantics::Role;
use view::{Cx, Kind, Listener, NodeId, StyleProp};

use super::{
    choice::Choice,
    metrics::ROW_GAP,
    navigation::{Step, step},
    row::{ListState, build_row},
};
use crate::kit::{INTER_FAMILY, use_theme};

/// The kind behind [`choice_list`](super::choice_list).
pub struct ChoiceListKind {
    pub(super) choices: Vec<Choice>,
    pub(super) selected: Signal<Option<usize>>,
}

fn step_for(key: NamedKey) -> Option<Step> {
    match key {
        NamedKey::ArrowUp | NamedKey::ArrowLeft => Some(Step::Previous),
        NamedKey::ArrowDown | NamedKey::ArrowRight => Some(Step::Next),
        _ => None,
    }
}

impl Kind for ChoiceListKind {
    const NAME: &'static str = "kit_choice_list";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let Self { choices, selected } = self;
        let count = choices.len();
        let shown: Memo<Option<usize>> =
            cx.memo(move |runtime| selected.get(runtime).filter(|index| *index < count));
        let disabled = cx.observe_disabled(node);

        cx.set_role(node, Role::Group);
        for prop in [
            StyleProp::FlexDirection(layouting::FlexDirection::Column),
            StyleProp::AlignSelf(Align::Stretch),
            StyleProp::Gap(None, Some(ROW_GAP)),
            StyleProp::Shrink(0.0),
            StyleProp::FontFamily(INTER_FAMILY.to_owned()),
        ] {
            cx.set_default_style(node, prop);
        }
        let titles: Vec<String> = choices.iter().map(|choice| choice.title.clone()).collect();
        cx.bind_value(node, move |runtime| {
            shown
                .get(runtime)
                .and_then(|index| titles.get(index).cloned())
                .unwrap_or_default()
        });

        let state = ListState {
            theme,
            selected,
            shown,
            disabled,
        };
        let rows: Vec<NodeId> = choices
            .iter()
            .enumerate()
            .map(|(index, choice)| build_row(cx, state, index, choice))
            .collect();
        cx.adopt_children(node, rows);

        let available: Vec<bool> = choices.iter().map(|choice| choice.available).collect();
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
                if let Some(next) = step(current, &available, direction) {
                    selected.set(cx, Some(next));
                }
            })),
        );
    }
}
