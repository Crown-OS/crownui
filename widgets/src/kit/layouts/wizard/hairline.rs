use semantics::Role;
use view::{Cx, Element, Kind, NodeId, StyleProp, Styled};

use crate::kit::{Theme, use_theme};
use crate::util::themed::bind_themed_style;

const HAIRLINE_THICKNESS: f32 = 1.0;

pub(super) struct HairlineKind;

impl Kind for HairlineKind {
    const NAME: &'static str = "kit_wizard_hairline";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        cx.set_role(node, Role::Separator);
        cx.set_default_style(node, StyleProp::Shrink(0.0));
        bind_themed_style(cx, node, theme, |theme: &Theme| {
            StyleProp::Background(Some(theme.surface.border))
        });
    }
}

pub(super) fn hairline() -> Element<HairlineKind> {
    Element::new(HairlineKind).h(HAIRLINE_THICKNESS)
}
