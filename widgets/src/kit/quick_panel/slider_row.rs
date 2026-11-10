use reactive::Signal;
use view::{Cx, Element, Kind, NodeId, Semantic, StyleProp, Styled, View};

use super::{
    badge::on_disc,
    metrics::{
        ICON_COLUMN, ICON_GAP, ICON_INSET, PAD_X, PLAIN_GLYPH, ROW_INSET, SLIDER_HEIGHT,
        SLIDER_OVERHANG,
    },
    palette::badge_idle,
    typography::{PanelGlyph, theme_color},
};
use crate::{
    kit::{slider, use_theme},
    stack::hstack,
};

/// The kind behind [`panel_slider`](super::panel_slider).
pub struct PanelSliderKind {
    pub(super) value: Signal<f32>,
    pub(super) label: String,
    pub(super) glyph: Option<PanelGlyph>,
}

/// The options of a [`panel_slider`](super::panel_slider).
pub trait PanelSliderOptions: Sized {
    /// The glyph on a disc at the slider's left, e.g. a speaker for volume.
    fn glyph(
        self,
        glyph: impl FnOnce(reactive::Memo<geometry::Color>) -> view::AnyView + 'static,
    ) -> Self;
}

impl PanelSliderOptions for Element<PanelSliderKind> {
    fn glyph(
        self,
        glyph: impl FnOnce(reactive::Memo<geometry::Color>) -> view::AnyView + 'static,
    ) -> Self {
        self.configure(|kind| kind.glyph = Some(Box::new(glyph)))
    }
}

impl Kind for PanelSliderKind {
    const NAME: &'static str = "kit_panel_slider";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        for prop in [
            StyleProp::FlexDirection(layouting::FlexDirection::Row),
            StyleProp::AlignItems(layouting::Align::Center),
            StyleProp::Padding(view::Edges::Left, ROW_INSET + ICON_INSET),
            StyleProp::Padding(view::Edges::Right, PAD_X),
            StyleProp::Size(
                view::SizeProperty::Height,
                layouting::Dimension::Length(SLIDER_HEIGHT),
            ),
            StyleProp::Shrink(0.0),
        ] {
            cx.set_default_style(node, prop);
        }
        let tint = theme_color(cx, theme, |theme| theme.text.body);
        let column = match self.glyph {
            Some(glyph) => on_disc(theme, glyph(tint), PLAIN_GLYPH, |_| badge_idle).boxed(),
            None => hstack(()).w(ICON_COLUMN).shrink(0.0).boxed(),
        };
        let track = slider(self.value)
            .label(self.label)
            .grow(1.0)
            .self_center()
            .ml(ICON_GAP - SLIDER_OVERHANG);
        cx.set_children(node, (column, track));
    }
}
