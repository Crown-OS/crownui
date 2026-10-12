use layouting::{Dimension, FlexDirection};
use view::{Cx, Edges, Element, IntoProp, Kind, NodeId, SizeProperty, StyleProp, Styled, View};

use super::{
    hairline::hairline,
    rail::{WizardRailKind, wizard_rail},
};
use crate::{
    kit::{INTER_FAMILY, use_theme},
    util::themed::bind_themed_style,
};

const FRAME_PADDING: f32 = 28.0;
const RAIL_GAP: f32 = 28.0;
const FOOTER_GAP: f32 = 16.0;

/// The kind behind [`wizard`].
pub struct WizardKind<Page, Footer> {
    rail: Element<WizardRailKind>,
    page: Page,
    footer: Footer,
}

impl<Page: View, Footer: View> Kind for WizardKind<Page, Footer> {
    const NAME: &'static str = "kit_wizard";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        cx.set_default_style(node, StyleProp::FlexDirection(FlexDirection::Column));
        cx.set_default_style(node, StyleProp::FontFamily(INTER_FAMILY.to_owned()));
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::Background(Some(theme.surface.bg))
        });

        let rail = cx.build(self.rail);
        cx.set_default_style(rail, StyleProp::Margin(Edges::Bottom, RAIL_GAP));
        let page = cx.build(self.page);
        for prop in [
            StyleProp::Grow(1.0),
            StyleProp::Shrink(1.0),
            StyleProp::Basis(Dimension::Length(0.0)),
            StyleProp::Size(SizeProperty::MinHeight, Dimension::Length(0.0)),
        ] {
            cx.set_default_style(page, prop);
        }
        let divider = cx.build(hairline().my(FOOTER_GAP));
        let footer = cx.build(self.footer);
        cx.adopt_children(node, vec![rail, page, divider, footer]);
    }
}

/// The wizard chrome: a step rail on top, the `page` filling the middle, and the `footer` button
/// bar pinned to the bottom under a hairline. It owns the frame, not the navigation.
pub fn wizard<Page: View, Footer: View>(
    steps: impl IntoIterator<Item = impl Into<String>>,
    current: impl IntoProp<usize>,
    page: Page,
    footer: Footer,
) -> Element<WizardKind<Page, Footer>> {
    Element::new(WizardKind {
        rail: wizard_rail(steps, current),
        page,
        footer,
    })
    .w_full()
    .h_full()
    .p(FRAME_PADDING)
}
