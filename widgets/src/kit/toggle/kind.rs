use geometry::{BLACK, Color, CornerRadii, Insets, Point, Rect, Vector, rgb};
use layouting::{Align, FlexDirection};
use reactive::Signal;
use semantics::Role;
use view::{Cx, IntoProp, Kind, Listener, NodeId, Prop, Shadow, StyleProp, Styled, View, bind};
use windowing::CursorIcon;

use super::metrics::{
    CAPTION_GAP, KNOB_GLOW_ALPHA, KNOB_GLOW_RADIUS, KNOB_HEIGHT, KNOB_PADDING, KNOB_RIM_ALPHA,
    KNOB_RIM_WIDTH, KNOB_TRAVEL, KNOB_WIDTH, TRACK_HEIGHT, TRACK_WIDTH,
};
use crate::{
    canvas::canvas,
    kit::{GradientStops, INTER_FAMILY, TEXT_SIZE, Theme, use_theme},
    stack::vstack,
    text::text,
    util::{color::scale_alpha, themed::themed},
};

const WHEAT: Color = rgb(0xf5deb3);

/// The kind behind [`toggle`](super::toggle).
pub struct ToggleKind {
    pub(super) on: Signal<bool>,
    pub(super) caption: Option<Prop<String>>,
}

fn gradient_layer(
    theme: Signal<Theme>,
    pick: impl Fn(&Theme) -> GradientStops + 'static,
    opacity: impl IntoProp<f32>,
) -> impl View {
    canvas(move |runtime, bounds, list| {
        let stops = theme.with(runtime, &pick);
        list.fill(
            Rect::new(Point::ZERO, bounds),
            CornerRadii::uniform(TRACK_HEIGHT / 2.0),
            stops.vertical(0.0, bounds.height),
        );
    })
    .absolute()
    .inset(0.0)
    .opacity(opacity)
}

fn knob(on: Signal<bool>, theme: Signal<Theme>) -> impl View {
    let rim = canvas(|_, bounds, list| {
        let half = KNOB_RIM_WIDTH / 2.0;
        list.stroke(
            Rect::new(Point::ZERO, bounds).inset(Insets::uniform(half)),
            CornerRadii::uniform(bounds.height / 2.0 - half),
            KNOB_RIM_WIDTH,
            scale_alpha(WHEAT, KNOB_RIM_ALPHA),
        );
    })
    .absolute()
    .inset(0.0);
    vstack(rim)
        .absolute()
        .left(KNOB_PADDING)
        .top(KNOB_PADDING)
        .w(KNOB_WIDTH)
        .h(KNOB_HEIGHT)
        .rounded(KNOB_HEIGHT / 2.0)
        .bg(themed(theme, |theme| theme.control.knob))
        .shadow(Shadow::new(
            scale_alpha(BLACK, KNOB_GLOW_ALPHA),
            KNOB_GLOW_RADIUS,
            Vector::ZERO,
        ))
        .translate(bind(move |runtime| {
            Vector::new(if on.get(runtime) { KNOB_TRAVEL } else { 0.0 }, 0.0)
        }))
}

impl Kind for ToggleKind {
    const NAME: &'static str = "kit_toggle";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let on = self.on;

        cx.set_role(node, Role::Switch);
        for prop in [
            StyleProp::FlexDirection(FlexDirection::Row),
            StyleProp::AlignItems(Align::Center),
            StyleProp::AlignSelf(Align::Start),
            StyleProp::Shrink(0.0),
            StyleProp::Cursor(CursorIcon::Pointer),
            StyleProp::TextSize(TEXT_SIZE),
            StyleProp::FontFamily(INTER_FAMILY.to_owned()),
        ] {
            cx.set_default_style(node, prop);
        }
        cx.add_listener(
            node,
            Listener::Click(Box::new(move |cx, _| {
                on.update(cx, |value| *value = !*value)
            })),
        );
        cx.bind_checked(node, move |runtime| on.get(runtime));

        let accent_opacity = bind(move |runtime| if on.get(runtime) { 1.0 } else { 0.0 });
        let track = vstack((
            gradient_layer(theme, |theme| theme.toggle_off, 1.0),
            gradient_layer(theme, |theme| theme.accent, accent_opacity),
            knob(on, theme),
        ))
        .relative()
        .w(TRACK_WIDTH)
        .h(TRACK_HEIGHT)
        .shrink(0.0)
        .rounded(TRACK_HEIGHT / 2.0);
        let track = cx.build(track);

        let mut children = vec![track];
        if let Some(caption) = self.caption {
            children.push(
                cx.build(
                    text(caption)
                        .ml(CAPTION_GAP)
                        .text_color(themed(theme, |theme| theme.text.body)),
                ),
            );
        }
        cx.adopt_children(node, children);
    }
}
