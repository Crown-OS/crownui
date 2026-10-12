mod hero_pane;
mod kind;
mod metrics;
mod step;

pub use hero_pane::hero_pane;
pub use kind::HeroWizardKind;
pub use metrics::{HERO_CARD_RADIUS, HERO_COLUMN_WIDTH};
pub use step::HeroStep;

use view::{Element, Styled, ViewSeq};

use crate::{kit::INTER_FAMILY, util::theme_fade::theme_fade};

/// A fixed-size split card centred on the page: `hero` on the left, then `step`'s heading, `content`, and `action` stretched across the foot of the column.
pub fn hero_wizard<Hero, Content, Action>(
    hero: Hero,
    step: HeroStep,
    content: Content,
    action: Action,
) -> Element<HeroWizardKind<Hero, Content, Action>>
where
    Hero: ViewSeq,
    Content: ViewSeq,
    Action: ViewSeq + Styled,
{
    Element::new(HeroWizardKind {
        hero,
        step,
        content,
        action: action.self_stretch(),
    })
    .flex_col()
    .justify_center()
    .w_full()
    .h_full()
    .font_family(INTER_FAMILY)
    .transition(theme_fade())
}
