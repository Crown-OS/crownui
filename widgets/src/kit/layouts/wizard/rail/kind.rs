use geometry::{CornerRadii, Rect};
use layouting::{FlexDirection, Position};
use semantics::Role;
use view::{Cx, Kind, NodeId, Prop, StyleProp, Styled};

use super::{
    stage::connector_color,
    step::{DOT_SIZE, step_column},
};
use crate::{
    canvas::canvas,
    kit::{INTER_FAMILY, use_theme},
};

const CONNECTOR_HEIGHT: f32 = 2.0;

/// The kind behind [`wizard_rail`](super::wizard_rail).
pub struct WizardRailKind {
    pub(super) labels: Vec<String>,
    pub(super) current: Prop<usize>,
}

impl Kind for WizardRailKind {
    const NAME: &'static str = "kit_wizard_rail";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let current = match self.current {
            Prop::Static(step) => cx.memo(move |_| step),
            Prop::Dynamic(mut compute) => cx.memo(move |runtime| compute(runtime)),
        };
        let steps = self.labels.len();

        cx.set_role(node, Role::ProgressBar);
        for prop in [
            StyleProp::FlexDirection(FlexDirection::Row),
            StyleProp::Position(Position::Relative),
            StyleProp::Shrink(0.0),
            StyleProp::FontFamily(INTER_FAMILY.to_owned()),
        ] {
            cx.set_default_style(node, prop);
        }
        let labels = self.labels.clone();
        cx.bind_value(node, move |runtime| {
            let step = current.get(runtime);
            labels.get(step).map_or_else(
                || "All steps complete".to_owned(),
                |label| format!("Step {} of {}: {}", step + 1, steps, label),
            )
        });

        let connectors = canvas(move |runtime, bounds, list| {
            let reached = current.get(runtime);
            let theme = theme.get(runtime);
            let column = bounds.width / steps as f32;
            let center = |step: usize| column * (step as f32 + 0.5);
            let top = (DOT_SIZE - CONNECTOR_HEIGHT) / 2.0;
            for step in 1..steps {
                let from = center(step - 1) + DOT_SIZE / 2.0;
                let to = center(step) - DOT_SIZE / 2.0;
                if to > from {
                    list.fill(
                        Rect::from_xywh(from, top, to - from, CONNECTOR_HEIGHT),
                        CornerRadii::ZERO,
                        connector_color(&theme, step <= reached),
                    );
                }
            }
        })
        .absolute()
        .inset(0.0);

        let mut layers = Vec::with_capacity(steps + 1);
        layers.push(cx.build(connectors));
        for (step, label) in self.labels.into_iter().enumerate() {
            layers.push(cx.build(step_column(theme, step, current, label)));
        }
        cx.adopt_children(node, layers);
    }
}
