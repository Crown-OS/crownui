use blinc_icons::icons;
use view::{Cx, Kind, NodeId, Prop, StyleProp, Styled};

use crate::{
    kit::{TEXT_SIZE, Tone, icon_colored, use_theme},
    stack::vstack,
    text::text,
    util::{
        color::scale_alpha,
        themed::{bind_themed_style, themed},
    },
};

pub(super) const RADIUS: f32 = 8.0;
pub(super) const PADDING: f32 = 12.0;
pub(super) const BORDER_WIDTH: f32 = 1.0;
pub(super) const ICON_GAP: f32 = 10.0;
const WASH_ALPHA: f32 = 0.10;
const BORDER_ALPHA: f32 = 0.32;
const ICON_SIZE: f32 = 16.0;
const ICON_STROKE: f32 = 2.0;
const TEXT_GAP: f32 = 3.0;
const BODY_TEXT_SIZE: f32 = TEXT_SIZE - 1.0;

const fn tone_icon(tone: Tone) -> &'static str {
    match tone {
        Tone::Success => icons::CIRCLE_CHECK,
        Tone::Warning => icons::TRIANGLE_ALERT,
        Tone::Danger => icons::OCTAGON_ALERT,
        Tone::Neutral => icons::INFO,
    }
}

/// The kind behind [`callout`](super::callout).
pub struct CalloutKind {
    pub(super) tone: Tone,
    pub(super) title: Prop<String>,
    pub(super) body: Option<Prop<String>>,
    pub(super) icon: Option<&'static str>,
}

impl Kind for CalloutKind {
    const NAME: &'static str = "kit_callout";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let tone = self.tone;
        bind_themed_style(cx, node, theme, move |theme| {
            StyleProp::Background(Some(scale_alpha(tone.color(theme), WASH_ALPHA)))
        });
        bind_themed_style(cx, node, theme, move |theme| {
            StyleProp::BorderColor(scale_alpha(tone.color(theme), BORDER_ALPHA))
        });

        let glyph = icon_colored(
            self.icon.unwrap_or(tone_icon(tone)),
            ICON_STROKE,
            move |theme| tone.color(theme),
        )
        .size(ICON_SIZE)
        .shrink(0.0);
        let title = text(self.title)
            .text_size(TEXT_SIZE)
            .text_color(themed(theme, |theme| theme.text.primary));
        let body = self.body.map(|body| {
            text(body)
                .text_size(BODY_TEXT_SIZE)
                .text_color(themed(theme, |theme| theme.text.muted))
        });
        let paragraphs = vstack((title, body)).gap(TEXT_GAP).items_start();
        cx.set_children(node, (glyph, paragraphs));
    }
}
