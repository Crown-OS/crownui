use geometry::Size;
use reactive::Signal;
use semantics::Role;
use view::{Cx, Element, Kind, Listener, NodeId, Semantic, Styled, bind, component, show};

use super::{
    handles::{OpenedBy, SelectHandles},
    keyboard::handle_key,
    metrics::*,
    panel::select_panel,
    skin::build_skin,
};
use crate::{
    stack::vstack,
    util::{anchored_popup::AnchoredPopup, interaction::Interaction},
};

/// The kind behind [`select`](super::select).
pub struct SelectKind {
    options: Signal<Vec<String>>,
    selected: Signal<Option<usize>>,
    pub(super) placeholder: String,
    pub(super) start_open: bool,
}

impl Kind for SelectKind {
    const NAME: &'static str = "kit_select";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let disabled = cx.observe_disabled(node);
        let interaction = Interaction::track(cx, node);
        let popup = AnchoredPopup::new(cx, node, Size::new(TRIGGER_WIDTH, TRIGGER_HEIGHT));
        let handles = SelectHandles {
            popup,
            options: self.options,
            placeholder: cx.signal(self.placeholder),
            selected: self.selected,
            highlighted: cx.signal(None),
            typed: cx.signal(String::new()),
            root: node,
        };
        cx.set_role(node, Role::ComboBox);
        cx.bind_value(node, move |runtime| {
            let selected = handles.selected.get(runtime);
            handles
                .options
                .with(runtime, |options| {
                    selected.and_then(|index| options.get(index).cloned())
                })
                .unwrap_or_default()
        });
        cx.effect(move |runtime| {
            handles.options.with(runtime, |_| ());
            handles.close(runtime);
        });
        cx.add_listener(
            node,
            Listener::Click(Box::new(move |cx, event| {
                let opened_by = if event.is_synthetic() {
                    OpenedBy::Keyboard
                } else {
                    OpenedBy::Pointer
                };
                if !disabled.get(cx) {
                    handles.toggle(cx, opened_by);
                }
            })),
        );
        cx.add_listener(
            node,
            Listener::Blur(Box::new(move |cx, _| handles.close(cx))),
        );
        cx.add_listener(
            node,
            Listener::KeyDown(Box::new(move |cx, event| {
                if !disabled.get(cx) {
                    handle_key(cx, event, handles);
                }
            })),
        );

        let skin = build_skin(cx, node, handles, interaction);
        let overlay = cx.build(
            show(
                popup.open_signal(),
                move || component(move |cx| popup.appear(cx, select_panel(handles))),
                || vstack(()),
            )
            .absolute()
            .left(0.0)
            .top(bind(move |runtime| popup.top(runtime))),
        );
        cx.adopt_children(node, vec![skin, overlay]);
        if self.start_open {
            handles.toggle(cx, OpenedBy::Pointer);
        }
    }
}

pub(super) fn select_element(
    options: Signal<Vec<String>>,
    selected: Signal<Option<usize>>,
) -> Element<SelectKind> {
    Element::new(SelectKind {
        options,
        selected,
        placeholder: String::new(),
        start_open: false,
    })
    .relative()
    .self_start()
    .w(TRIGGER_WIDTH)
    .h(TRIGGER_HEIGHT)
    .shrink(0.0)
    .focusable(true)
}
