use geometry::{CornerRadii, Insets, Point, Rect};
use view::{Corners, Cx, Kind, NodeId, StyleProp, Styled, ViewSeq};

use super::{fill::PaneFill, radii::Radii};
use crate::{
    canvas::canvas,
    kit::{Theme, use_theme},
};

pub(super) type FillSource = Box<dyn Fn(&Theme) -> PaneFill>;
pub(super) type BorderSource = Box<dyn Fn(&Theme) -> geometry::Color>;

/// The kind behind [`pane`](super::pane).
pub struct PaneKind<C> {
    pub(super) content: C,
    pub(super) fill: FillSource,
    pub(super) radii: Radii,
    pub(super) border: Option<(BorderSource, f32)>,
}

impl<C: ViewSeq> Kind for PaneKind<C> {
    const NAME: &'static str = "kit_pane";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let Self {
            content,
            fill,
            radii,
            border,
        } = self;
        for (corners, radius) in [
            (Corners::TopLeft, radii.top_left),
            (Corners::TopRight, radii.top_right),
            (Corners::BottomRight, radii.bottom_right),
            (Corners::BottomLeft, radii.bottom_left),
        ] {
            cx.set_default_style(node, StyleProp::Radius(corners, radius));
        }
        cx.set_default_style(node, StyleProp::Position(layouting::Position::Relative));
        cx.set_default_style(node, StyleProp::Overflow(layouting::Overflow::Clip));

        let backdrop = canvas(move |runtime, bounds, list| {
            let rect = Rect::new(Point::ZERO, bounds);
            theme.with(runtime, |theme| {
                if let Some(paint) = fill(theme).paint(bounds.height) {
                    list.fill(rect, CornerRadii::from(radii), paint);
                }
                if let Some((color, width)) = &border {
                    let half = width / 2.0;
                    list.stroke(
                        rect.inset(Insets::uniform(half)),
                        radii.inset(half),
                        *width,
                        color(theme),
                    );
                }
            });
        })
        .absolute()
        .inset(0.0);
        let backdrop = cx.build(backdrop);
        let mut children = vec![backdrop];
        content.build_into(cx, &mut children);
        cx.adopt_children(node, children);
    }
}
