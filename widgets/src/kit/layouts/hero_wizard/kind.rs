use view::{Cx, Kind, NodeId, StyleProp, Styled, ViewSeq};

use super::{
    metrics::{
        ACTION_GAP, BAR_GAP, CARD_HEIGHT, COLUMN_PADDING, HERO_CARD_RADIUS, HERO_COLUMN_WIDTH,
    },
    step::HeroStep,
};
use crate::{
    kit::{PaneFill, PaneOptions, Radii, pane, soft_shadow, use_theme},
    spacer::spacer,
    stack::vstack,
    util::themed::bind_themed_style,
};

/// The kind behind [`hero_wizard`](super::hero_wizard).
pub struct HeroWizardKind<Hero, Content, Action> {
    pub(super) hero: Hero,
    pub(super) step: HeroStep,
    pub(super) content: Content,
    pub(super) action: Action,
}

impl<Hero: ViewSeq, Content: ViewSeq, Action: ViewSeq> Kind
    for HeroWizardKind<Hero, Content, Action>
{
    const NAME: &'static str = "kit_hero_wizard";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::Background(Some(theme.surface.bg))
        });

        let column = vstack((
            self.step.header(theme).mb(BAR_GAP),
            vstack(self.content).shrink(0.0),
            spacer().min_h(ACTION_GAP),
            self.action,
        ))
        .w(HERO_COLUMN_WIDTH)
        .h(CARD_HEIGHT - COLUMN_PADDING * 2.0);
        let card = pane((self.hero, vstack(column).p(COLUMN_PADDING).shrink(0.0)))
            .themed_fill(|theme| PaneFill::Color(theme.surface.raised))
            .radii(Radii::all(HERO_CARD_RADIUS))
            .flex_row()
            .items_center();
        cx.set_children(node, soft_shadow(card, HERO_CARD_RADIUS).self_center());
    }
}
