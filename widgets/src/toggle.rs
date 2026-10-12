use geometry::{Color, Vector};
use reactive::Signal;
use semantics::Role;
use view::{Corners, Cx, Element, Kind, Listener, NodeId, StyleProp, Styled, bind};
use windowing::CursorIcon;

const TRACK_WIDTH: f32 = 44.0;
const TRACK_HEIGHT: f32 = 24.0;
const KNOB_INSET: f32 = 2.0;
const KNOB_SIZE: f32 = TRACK_HEIGHT - 2.0 * KNOB_INSET;
const ON_COLOR: Color = Color::new(0.2, 0.78, 0.35, 1.0);
const OFF_COLOR: Color = Color::new(0.5, 0.5, 0.5, 0.35);

/// The kind behind [`toggle`].
pub struct ToggleKind {
    on: Signal<bool>,
}

impl Kind for ToggleKind {
    const NAME: &'static str = "toggle";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let on = self.on;
        cx.set_role(node, Role::Switch);
        for prop in [
            StyleProp::Size(
                view::SizeProperty::Width,
                layouting::Dimension::Length(TRACK_WIDTH),
            ),
            StyleProp::Size(
                view::SizeProperty::Height,
                layouting::Dimension::Length(TRACK_HEIGHT),
            ),
            StyleProp::Radius(Corners::All, TRACK_HEIGHT / 2.0),
            StyleProp::Padding(view::Edges::All, KNOB_INSET),
            StyleProp::Shrink(0.0),
            StyleProp::Cursor(CursorIcon::Pointer),
        ] {
            cx.set_default_style(node, prop);
        }
        let knob = view::Element::new(KnobKind)
            .size(KNOB_SIZE)
            .rounded(KNOB_SIZE / 2.0)
            .bg(geometry::WHITE)
            .translate(bind(move |runtime| {
                let travel = TRACK_WIDTH - KNOB_SIZE - 2.0 * KNOB_INSET;
                Vector::new(if on.get(runtime) { travel } else { 0.0 }, 0.0)
            }));
        cx.set_children(node, knob);
        cx.add_listener(
            node,
            Listener::Click(Box::new(move |cx, _event| {
                on.update(cx, |value| *value = !*value);
            })),
        );
        cx.bind_style(node, move |runtime| {
            StyleProp::Background(Some(if on.get(runtime) { ON_COLOR } else { OFF_COLOR }))
        });
        cx.bind_checked(node, move |runtime| on.get(runtime));
    }
}

struct KnobKind;

impl Kind for KnobKind {
    const NAME: &'static str = "knob";

    fn mount(self, _cx: &mut Cx, _node: NodeId) {}
}

/// A switch bound to `on`; clicking or pressing Space flips it, and the knob animates when an
/// ancestor (or the toggle itself) has `.transition(..)`.
pub fn toggle(on: Signal<bool>) -> Element<ToggleKind> {
    Element::new(ToggleKind { on }).transition(motion::Spring::snappy())
}
