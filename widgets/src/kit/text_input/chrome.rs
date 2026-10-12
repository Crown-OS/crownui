use geometry::{CornerRadii, Insets, Point, Rect, Size};
use graphics::{DrawList, Paint};
use reactive::{Runtime, Signal};
use view::{Cx, NodeId, Styled, bind};

use crate::{canvas::canvas, kit::Theme, util::spring};

pub(crate) fn inset_stroke(
    list: &mut DrawList,
    bounds: Size,
    radius: f32,
    width: f32,
    paint: impl Into<Paint>,
) {
    let inset = width / 2.0;
    list.stroke(
        Rect::new(Point::ZERO, bounds).inset(Insets::uniform(inset)),
        CornerRadii::uniform(radius - inset),
        width,
        paint,
    );
}

pub(crate) fn mount_chrome(
    cx: &mut Cx,
    theme: Signal<Theme>,
    radius: f32,
    ring: impl Fn(&mut Runtime) -> f32 + 'static,
) -> [NodeId; 2] {
    let surface = canvas(move |runtime, bounds, list| {
        let colors = theme.with(runtime, |theme| theme.popover);
        list.fill(
            Rect::new(Point::ZERO, bounds),
            CornerRadii::uniform(radius),
            colors.trigger_bg,
        );
        inset_stroke(list, bounds, radius, 1.0, colors.border);
    });
    let edge = canvas(move |runtime, bounds, list| {
        let ring = theme.with(runtime, |theme| theme.popover.ring);
        inset_stroke(list, bounds, radius, 1.0, ring.vertical(0.0, bounds.height));
    })
    .opacity(bind(ring))
    .transition(spring::snappy());
    [
        cx.build(surface.absolute().inset(0.0)),
        cx.build(edge.absolute().inset(0.0)),
    ]
}
