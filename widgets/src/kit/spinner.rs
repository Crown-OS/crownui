use std::f32::consts::FRAC_PI_2;

use geometry::{CornerRadii, Point, Rect};
use graphics::{DrawCommand, Paint, Stroke};
use motion::Curve;
use semantics::Role;
use view::{Cx, Element, Kind, NodeId, Styled};

use super::theme::use_theme;
use crate::{
    canvas::canvas,
    util::{arc::arc_path, color::scale_alpha, spring},
};

const DEFAULT_DIAMETER: f32 = 15.0;
const STROKE_RATIO: f32 = 0.14;
const TURNS_PER_SECOND: f32 = 0.85;
const ARC_SWEEP: f32 = FRAC_PI_2;
const TRACK_ALPHA: f32 = 0.55;
const FULL_TURN_DEGREES: f32 = 360.0;
const TURN_MILLIS: u64 = (1000.0 / TURNS_PER_SECOND) as u64;

/// The kind behind [`spinner`].
pub struct SpinnerKind;

impl Kind for SpinnerKind {
    const NAME: &'static str = "kit_spinner";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        cx.set_role(node, Role::ProgressBar);

        let track = canvas(move |runtime, bounds, list| {
            let width = (bounds.width.min(bounds.height) * STROKE_RATIO).max(1.0);
            let diameter = bounds.width.min(bounds.height);
            let ring =
                Rect::from_xywh(width / 2.0, width / 2.0, diameter - width, diameter - width);
            let color = theme.with(runtime, |theme| theme.control.track);
            list.stroke(
                ring,
                CornerRadii::uniform(ring.size.width / 2.0),
                width,
                scale_alpha(color, TRACK_ALPHA),
            );
        })
        .absolute()
        .inset(0.0);

        let head = canvas(move |runtime, bounds, list| {
            let diameter = bounds.width.min(bounds.height);
            let width = (diameter * STROKE_RATIO).max(1.0);
            let radius = (diameter - width) / 2.0;
            let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
            let color = theme.with(runtime, |theme| theme.accent.end);
            list.push(DrawCommand::StrokePath(
                arc_path(center, radius, 0.0, ARC_SWEEP),
                Stroke::rounded(width),
                Paint::solid(color),
            ));
        })
        .absolute()
        .inset(0.0)
        .rotate(FULL_TURN_DEGREES)
        .enter(|style| style.rotate(0.0))
        .transition(Curve::linear(TURN_MILLIS).repeating());

        let layers = vec![cx.build(track), cx.build(head)];
        cx.adopt_children(node, layers);
    }
}

/// An indeterminate activity indicator: a ring with a rotating accent arc, 15px across; resize it
/// with `.size(..)` and the stroke scales with it. Mount it only while work is in flight.
pub fn spinner() -> Element<SpinnerKind> {
    Element::new(SpinnerKind)
        .size(DEFAULT_DIAMETER)
        .shrink(0.0)
        .transition(spring::snappy())
        .enter(|style| style.opacity(0.0))
}
