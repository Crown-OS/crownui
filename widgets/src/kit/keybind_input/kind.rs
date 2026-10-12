use crownconfig::Keybind;
use layouting::{Align, Dimension, FlexDirection, Justify, Overflow, Position};
use reactive::Signal;
use semantics::Role;
use view::{
    Corners, Cx, Element, Kind, Listener, NodeId, SizeProperty, StyleProp, Styled, for_each,
};

use super::{
    cap::piece,
    metrics::{CAP_GAP, RECORDING_HINT},
    recorder::{KeyPhase, Outcome, Recorder},
};
use crate::{
    kit::{
        INTER_FAMILY, TEXT_SIZE,
        text_input::{DEFAULT_WIDTH, FIELD_HEIGHT, FIELD_RADIUS, H_PADDING, mount_chrome},
        use_theme,
    },
    stack::hstack,
};

/// The kind behind [`keybind_input`](super::keybind_input).
pub struct KeybindInputKind {
    pub(super) value: Signal<Keybind>,
    pub(super) width: f32,
}

fn capture(
    cx: &mut Cx,
    node: NodeId,
    value: Signal<Keybind>,
    recorder: Signal<Recorder>,
    event: &mut view::Event<input::KeyboardEvent>,
    phase: KeyPhase,
) {
    if !recorder.with(cx, Recorder::is_recording) {
        return;
    }
    let mut state = recorder.get(cx);
    event.prevent_default();
    event.stop_propagation();
    if event.repeat {
        return;
    }
    match state.on_key(event, phase) {
        Outcome::Commit(bind) => {
            state.disarm();
            value.set(cx, bind);
            cx.emit_change(node, bind.to_string());
        }
        Outcome::Cancel => state.disarm(),
        Outcome::Pending | Outcome::Ignored => {}
    }
    recorder.set(cx, state);
}

impl Kind for KeybindInputKind {
    const NAME: &'static str = "kit_keybind_input";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let disabled = cx.observe_disabled(node);
        let focused = cx.signal(false);
        let recorder = cx.signal(Recorder::default());
        let value = self.value;

        cx.set_role(node, Role::TextField);
        cx.bind_value(node, move |runtime| {
            if recorder.with(runtime, Recorder::is_recording) {
                RECORDING_HINT.to_owned()
            } else {
                value.get(runtime).to_string()
            }
        });
        for prop in [
            StyleProp::FlexDirection(FlexDirection::Row),
            StyleProp::AlignItems(Align::Center),
            StyleProp::AlignSelf(Align::Start),
            StyleProp::Position(Position::Relative),
            StyleProp::Size(SizeProperty::Width, Dimension::Length(self.width)),
            StyleProp::Size(SizeProperty::Height, Dimension::Length(FIELD_HEIGHT)),
            StyleProp::Overflow(Overflow::Clip),
            StyleProp::Radius(Corners::All, FIELD_RADIUS),
            StyleProp::Shrink(0.0),
            StyleProp::TextSize(TEXT_SIZE),
            StyleProp::FontFamily(INTER_FAMILY.to_owned()),
        ] {
            cx.set_default_style(node, prop);
        }

        let pieces = cx.signal(Vec::new());
        cx.effect(move |runtime| {
            let bound = value.get(runtime);
            let next = recorder.with(runtime, |recorder| recorder.pieces(bound));
            pieces.set(runtime, next);
        });
        cx.effect(move |runtime| {
            value.get(runtime);
            recorder.update(runtime, Recorder::disarm);
        });

        let ring = move |runtime: &mut reactive::Runtime| {
            if focused.get(runtime) && !disabled.get(runtime) {
                1.0
            } else {
                0.0
            }
        };
        let mut layers = mount_chrome(cx, theme, FIELD_RADIUS, ring).to_vec();
        let caps = for_each(
            pieces,
            |piece| piece.clone(),
            move |cx, item| piece(theme, disabled, item.with_untracked(cx, Clone::clone)),
        );
        let row = cx.build(caps.flex_row().gap(CAP_GAP).items_center().shrink(0.0));
        let viewport = cx.build(
            hstack(())
                .absolute()
                .left(H_PADDING)
                .right(H_PADDING)
                .top(0.0)
                .bottom(0.0)
                .items_center()
                .overflow_hidden(),
        );
        cx.adopt_children(viewport, vec![row]);
        let (row_size, viewport_size) = (cx.observe_size(row), cx.observe_size(viewport));
        cx.bind_style(viewport, move |runtime| {
            StyleProp::Justify(
                if row_size.get(runtime).width > viewport_size.get(runtime).width {
                    Justify::Start
                } else {
                    Justify::Center
                },
            )
        });
        layers.push(viewport);
        cx.adopt_children(node, layers);

        cx.add_listener(
            node,
            Listener::Focus(Box::new(move |cx, _| focused.set(cx, true))),
        );
        cx.add_listener(
            node,
            Listener::Blur(Box::new(move |cx, _| {
                focused.set(cx, false);
                if recorder.with(cx, Recorder::is_recording) {
                    recorder.update(cx, Recorder::disarm);
                }
            })),
        );
        cx.add_listener(
            node,
            Listener::Click(Box::new(move |cx, _| {
                if recorder.with(cx, Recorder::is_recording) {
                    recorder.update(cx, Recorder::disarm);
                } else {
                    cx.request_focus(node);
                    recorder.update(cx, Recorder::arm);
                }
            })),
        );
        cx.add_listener(
            node,
            Listener::KeyDown(Box::new(move |cx, event| {
                capture(cx, node, value, recorder, event, KeyPhase::Down);
            })),
        );
        cx.add_listener(
            node,
            Listener::KeyUp(Box::new(move |cx, event| {
                capture(cx, node, value, recorder, event, KeyPhase::Up);
            })),
        );
    }
}

/// A field that records a keyboard shortcut: click it (or press Enter on it), then strike the
/// chord. Listen with `.on_change`, which receives the chord as the config file writes it
/// (`Super+Space`, or `None` once cleared).
pub fn keybind_input(value: Signal<Keybind>) -> Element<KeybindInputKind> {
    Element::new(KeybindInputKind {
        value,
        width: DEFAULT_WIDTH,
    })
}
