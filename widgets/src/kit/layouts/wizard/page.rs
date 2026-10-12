use layouting::FlexDirection;
use view::{Cx, Element, IntoProp, Kind, NodeId, Prop, StyleProp, Styled, View};

use crate::{
    kit::{INTER_FAMILY, TEXT_SIZE, use_theme},
    stack::vstack,
    text::text,
    util::themed::themed,
};

const HEADER_GAP: f32 = 20.0;
const TITLE_GAP: f32 = 6.0;
const TITLE_TEXT_SIZE: f32 = TEXT_SIZE + 7.0;
const SUBTITLE_TEXT_SIZE: f32 = TEXT_SIZE;

/// The kind behind [`wizard_page`].
pub struct WizardPageKind<Content> {
    title: Prop<String>,
    subtitle: Option<Prop<String>>,
    content: Content,
}

impl<Content: View> Kind for WizardPageKind<Content> {
    const NAME: &'static str = "kit_wizard_page";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        cx.set_default_style(node, StyleProp::FlexDirection(FlexDirection::Column));
        cx.set_default_style(node, StyleProp::FontFamily(INTER_FAMILY.to_owned()));

        let title = text(self.title)
            .text_size(TITLE_TEXT_SIZE)
            .text_color(themed(theme, |theme| theme.text.primary));
        let subtitle = self.subtitle.map(|subtitle| {
            text(subtitle)
                .text_size(SUBTITLE_TEXT_SIZE)
                .text_color(themed(theme, |theme| theme.text.muted))
        });
        let header = vstack((title, subtitle)).gap(TITLE_GAP).items_start();
        let content = cx.build(self.content);
        cx.set_default_style(content, StyleProp::Margin(view::Edges::Top, HEADER_GAP));
        let header = cx.build(header);
        cx.adopt_children(node, vec![header, content]);
    }
}

/// A titled step body: a heading, an optional line of explanation, and the step's own controls.
pub fn wizard_page<Content: View>(
    title: impl IntoProp<String>,
    subtitle: Option<impl IntoProp<String>>,
    content: Content,
) -> Element<WizardPageKind<Content>> {
    Element::new(WizardPageKind {
        title: title.into_prop(),
        subtitle: subtitle.map(IntoProp::into_prop),
        content,
    })
}
