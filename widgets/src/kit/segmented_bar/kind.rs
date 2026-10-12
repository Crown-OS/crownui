use geometry::{CornerRadii, Point, Rect, Size};
use semantics::Role;
use view::{Cx, Kind, NodeId, Prop};

use super::segment::Segment;
use crate::{kit::use_theme, util::shared_prop::shared_prop};

/// The kind behind [`segmented_bar`](super::segmented_bar).
pub struct SegmentedBarKind {
    pub(super) segments: Prop<Vec<Segment>>,
}

impl Kind for SegmentedBarKind {
    const NAME: &'static str = "kit_segmented_bar";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let segments = shared_prop(cx, self.segments);
        cx.set_role(node, Role::Group);
        cx.set_canvas(node, move |runtime, bounds, list| {
            let track = theme.with(runtime, |theme| theme.control.track);
            let bar = Rect::new(Point::ZERO, bounds);
            let pill = CornerRadii::uniform(bounds.height / 2.0);
            segments.with(runtime, |segments| {
                let total: f32 = segments.iter().map(Segment::usable_weight).sum();
                if total <= 0.0 || bounds.width <= 0.0 {
                    list.fill(bar, pill, track);
                    return;
                }
                list.save();
                list.clip(bar, pill);
                let mut left = 0.0;
                for (index, run) in segments.iter().enumerate() {
                    let right = if index + 1 == segments.len() {
                        bounds.width
                    } else {
                        left + run.usable_weight() / total * bounds.width
                    };
                    if right > left {
                        let span = Rect::new(
                            Point::new(left, 0.0),
                            Size::new(right - left, bounds.height),
                        );
                        list.fill(span, CornerRadii::ZERO, run.color);
                    }
                    left = right;
                }
                list.restore();
            });
        });
    }
}
