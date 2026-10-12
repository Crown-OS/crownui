use geometry::{CornerRadii, Point, Rect};
use semantics::Role;
use view::{Cx, Kind, NodeId, Prop, StyleProp, Styled};

use crate::{
    canvas::canvas,
    kit::use_theme,
    util::{shared_prop::shared_prop, themed::bind_themed_style},
};

pub(super) const HEIGHT: f32 = 8.0;
pub(super) const DEFAULT_WIDTH: f32 = 240.0;

fn clamp_fraction(fraction: f32) -> f32 {
    if fraction.is_nan() {
        0.0
    } else {
        fraction.clamp(0.0, 1.0)
    }
}

/// The kind behind [`progress`](super::progress).
pub struct ProgressKind {
    pub(super) fraction: Prop<f32>,
}

impl Kind for ProgressKind {
    const NAME: &'static str = "kit_progress";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let fraction = shared_prop(cx, self.fraction.map(clamp_fraction));
        cx.set_role(node, Role::ProgressBar);
        cx.bind_value(node, move |runtime| {
            format!("{}%", (fraction.get(runtime) * 100.0).round())
        });
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::Background(Some(theme.control.track))
        });

        let fill = canvas(move |runtime, bounds, list| {
            let filled_width = fraction.get(runtime) * bounds.width;
            if filled_width < HEIGHT {
                return;
            }
            let accent = theme.with(runtime, |theme| theme.accent);
            list.fill(
                Rect::new(
                    Point::ZERO,
                    geometry::Size::new(filled_width, bounds.height),
                ),
                CornerRadii::uniform(HEIGHT / 2.0),
                accent.horizontal(0.0, bounds.width),
            );
        })
        .absolute()
        .inset(0.0);
        cx.set_children(node, fill);
    }
}
