use geometry::{Color, CornerRadii, Point, Rect};
use layouting::{Dimension, Position};
use reactive::{Memo, Signal};
use view::{Cx, IntoProp, Kind, NodeId, SizeProperty, StyleProp, Styled, View, bind};

use crate::{
    canvas::canvas,
    kit::{GradientStops, Theme, use_theme},
    stack::vstack,
    util::color::scale_alpha,
};

const RIM_WIDTH: f32 = 1.0;
const SELECTED_RING_WIDTH: f32 = 5.0;
const DIMMED_ALPHA: f32 = 0.4;

/// The kit's radio circle: a gradient ring around a hole that shrinks from a 1px rim to a 5px
/// ring as `selected` turns on, animated by the nearest ancestor's transition.
pub(crate) struct RadioMarkKind {
    pub(crate) diameter: f32,
    pub(crate) selected: Memo<bool>,
    pub(crate) dimmed: Memo<bool>,
}

pub(crate) fn muted(color: Color, dimmed: bool) -> Color {
    if dimmed {
        scale_alpha(color, DIMMED_ALPHA)
    } else {
        color
    }
}

fn ring_layer(
    theme: Signal<Theme>,
    dimmed: Memo<bool>,
    pick: fn(&Theme) -> GradientStops,
    opacity: impl IntoProp<f32>,
) -> impl View {
    canvas(move |runtime, bounds, list| {
        let is_dimmed = dimmed.get(runtime);
        let stops = theme.with(runtime, |theme| {
            pick(theme).map(|color| muted(color, is_dimmed))
        });
        list.fill(
            Rect::new(Point::ZERO, bounds),
            CornerRadii::uniform(bounds.height / 2.0),
            stops.vertical(0.0, bounds.height),
        );
    })
    .absolute()
    .inset(0.0)
    .opacity(opacity)
}

impl Kind for RadioMarkKind {
    const NAME: &'static str = "kit_radio_mark";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let Self {
            diameter,
            selected,
            dimmed,
        } = self;
        let radius = diameter / 2.0;
        for prop in [
            StyleProp::Size(SizeProperty::Width, Dimension::Length(diameter)),
            StyleProp::Size(SizeProperty::Height, Dimension::Length(diameter)),
            StyleProp::Radius(view::Corners::All, radius),
            StyleProp::Position(Position::Relative),
            StyleProp::Shrink(0.0),
        ] {
            cx.set_default_style(node, prop);
        }

        let rim = ring_layer(
            theme,
            dimmed,
            |theme| GradientStops::flat(theme.radio.unselected_border),
            bind(move |runtime| {
                if dimmed.get(runtime) && selected.get(runtime) {
                    0.0
                } else {
                    1.0
                }
            }),
        );
        let accent = ring_layer(
            theme,
            dimmed,
            |theme| theme.radio.accent,
            bind(move |runtime| if selected.get(runtime) { 1.0 } else { 0.0 }),
        );
        let hole_diameter = diameter - 2.0 * RIM_WIDTH;
        let hole = vstack(())
            .absolute()
            .left(RIM_WIDTH)
            .top(RIM_WIDTH)
            .size(hole_diameter)
            .rounded(hole_diameter / 2.0)
            .bg(bind(move |runtime| {
                let (is_selected, is_dimmed) = (selected.get(runtime), dimmed.get(runtime));
                theme.with(runtime, |theme| {
                    let color = if is_selected {
                        theme.radio.hole
                    } else {
                        theme.radio.unselected_bg
                    };
                    muted(color, is_dimmed)
                })
            }))
            .scale(bind(move |runtime| {
                if selected.get(runtime) {
                    (radius - SELECTED_RING_WIDTH) / (radius - RIM_WIDTH)
                } else {
                    1.0
                }
            }));
        cx.set_children(node, (rim, accent, hole));
    }
}
