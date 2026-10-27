use input::{Key, NamedKey};
use layouting::{Align, FlexDirection, Position};
use reactive::Signal;
use semantics::Role;
use view::{Cx, Element, Kind, Listener, NodeId, Semantic, StyleProp, Styled, bind, show};

use super::{
    DEFAULT_PLACEHOLDER,
    metrics::{DEFAULT_MAX_ROWS, PANEL_GAP},
    picker::{Picker, SelectHandler},
    popup::panel,
    state::{SearchState, Step},
    suggestion::Suggestion,
};
use crate::{
    kit::{
        ClearButton, FieldShape, INTER_FAMILY, TEXT_SIZE, TextInputKind, icons,
        text_input::{FIELD_HEIGHT, FieldWidth},
        use_theme,
    },
    stack::vstack,
    util::text_edit::Selection,
};

/// The kind behind [`search`](super::search).
pub struct SearchKind {
    pub(super) input: TextInputKind,
    pub(super) suggestions: Option<Signal<Vec<Suggestion>>>,
    pub(super) max_suggestions: usize,
    pub(super) on_select: Option<SelectHandler>,
}

impl SearchKind {
    pub(super) fn new(query: Signal<String>) -> Self {
        let mut input = TextInputKind::new(query);
        input.leading_icon = Some(icons::SEARCH);
        input.placeholder = DEFAULT_PLACEHOLDER.to_owned();
        input.shape = FieldShape::Pill;
        input.clear_button = ClearButton::Visible;
        input.role = Role::SearchField;
        Self {
            input,
            suggestions: None,
            max_suggestions: DEFAULT_MAX_ROWS,
            on_select: None,
        }
    }
}

impl Kind for SearchKind {
    const NAME: &'static str = "kit_search";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let Self {
            mut input,
            suggestions,
            max_suggestions,
            on_select,
        } = self;
        let theme = use_theme(cx);
        let disabled = cx.observe_disabled(node);
        let state = SearchState::new(cx);
        let query = input.value;
        let initial_end = query.with(cx, String::len);
        let selection = cx.signal(Selection::at(initial_end));
        input.selection = Some(selection);
        let align = match input.width {
            FieldWidth::Fixed(_) => Align::Start,
            FieldWidth::Fill => Align::Stretch,
        };
        let placeholder = input.placeholder.clone();

        cx.set_role(node, Role::ComboBox);
        for prop in [
            StyleProp::FlexDirection(FlexDirection::Column),
            StyleProp::Position(Position::Relative),
            StyleProp::AlignSelf(align),
            StyleProp::Shrink(0.0),
            StyleProp::TextSize(TEXT_SIZE),
            StyleProp::FontFamily(INTER_FAMILY.to_owned()),
        ] {
            cx.set_default_style(node, prop);
        }

        let source = suggestions.unwrap_or_else(|| cx.signal(Vec::new()));
        cx.effect(move |runtime| {
            let rows = source.with(runtime, |rows| {
                rows.iter()
                    .take(max_suggestions)
                    .cloned()
                    .enumerate()
                    .collect()
            });
            state.rows.set(runtime, rows);
            state.highlight.set(runtime, None);
        });

        let field = cx.build(Element::new(input).label(placeholder).disabled(disabled));
        let picker = Picker {
            field,
            query,
            selection,
            state,
            on_select: cx.store(on_select),
        };
        cx.add_listener(
            field,
            Listener::Focus(Box::new(move |cx, _| state.focus_changed(cx, true))),
        );
        cx.add_listener(
            field,
            Listener::Blur(Box::new(move |cx, _| state.focus_changed(cx, false))),
        );
        cx.add_listener(
            field,
            Listener::Input(Box::new(move |cx, _| state.edited(cx))),
        );
        cx.add_listener(
            field,
            Listener::Submit(Box::new(move |cx, event| {
                let highlighted = state.highlight.get(cx);
                state.dismiss(cx);
                if let Some(index) = highlighted {
                    event.stop_propagation();
                    picker.pick(cx, index);
                }
            })),
        );
        cx.add_listener(
            node,
            Listener::KeyDown(Box::new(move |cx, event| match event.key {
                Key::Named(NamedKey::Escape) if state.is_open(cx) => {
                    state.dismiss(cx);
                    event.stop_propagation();
                }
                Key::Named(NamedKey::ArrowDown) => state.step(cx, Step::Next),
                Key::Named(NamedKey::ArrowUp) => state.step(cx, Step::Previous),
                _ => {}
            })),
        );

        let popup = cx.build(
            show(
                bind(move |runtime| state.is_open(runtime)),
                move || panel(theme, state, picker),
                || vstack(()),
            )
            .absolute()
            .left(0.0)
            .right(0.0)
            .top(FIELD_HEIGHT + PANEL_GAP)
            .top_layer(),
        );
        cx.adopt_children(node, vec![field, popup]);
    }
}

/// A search bar: a pill-shaped text input with a magnifier, a clear button and a popup of
/// suggestions that follows what is typed. The popup is up while the bar is focused and there
/// are suggestions; Up and Down walk the rows, Enter picks the highlighted one (or submits the
/// query), Escape puts the popup away until the next edit.
pub fn search(query: Signal<String>) -> Element<SearchKind> {
    Element::new(SearchKind::new(query))
}
