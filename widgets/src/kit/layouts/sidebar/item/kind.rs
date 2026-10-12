use graphics::FontWeight;
use layouting::{Align, Dimension, FlexDirection, Position};
use view::{Cx, Edges, Kind, NodeId, Prop, SizeProperty, StyleProp, Styled};

use super::{
    layers::{fixed_icon, hover_wash, leading_icon, selected_pill},
    variant::{
        HORIZONTAL_PADDING, ICON_GAP, LEADING_STROKE, SidebarItemVariant, TRAILING_SIZE,
        TRAILING_STROKE,
    },
};
use crate::{
    kit::{INTER_FAMILY, use_theme},
    text::text,
    util::interaction::Interaction,
};

/// The kind behind [`sidebar_item`](super::sidebar_item) and its siblings.
pub struct SidebarItemKind {
    pub(super) variant: SidebarItemVariant,
    pub(super) label: Prop<String>,
    pub(super) icon: Option<&'static str>,
    pub(super) trailing_icon: Option<&'static str>,
    pub(super) selected: Prop<bool>,
}

impl Kind for SidebarItemKind {
    const NAME: &'static str = "kit_sidebar_item";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let disabled = cx.observe_disabled(node);
        let interaction = Interaction::track(cx, node);
        let variant = self.variant;
        let selected = match self.selected {
            Prop::Static(value) => cx.memo(move |_| value),
            Prop::Dynamic(mut compute) => cx.memo(move |runtime| compute(runtime)),
        };

        cx.set_role(node, variant.role());
        for prop in [
            StyleProp::FlexDirection(FlexDirection::Row),
            StyleProp::AlignItems(Align::Center),
            StyleProp::Position(Position::Relative),
            StyleProp::Size(SizeProperty::Height, Dimension::Length(variant.height())),
            StyleProp::Padding(Edges::Left, variant.leading_padding()),
            StyleProp::Padding(Edges::Right, HORIZONTAL_PADDING),
            StyleProp::Shrink(0.0),
            StyleProp::TextSize(variant.text_size()),
            StyleProp::FontFamily(INTER_FAMILY.to_owned()),
            StyleProp::FontWeight(FontWeight::MEDIUM),
        ] {
            cx.set_default_style(node, prop);
        }
        cx.bind_style(node, move |runtime| {
            let chosen = selected.get(runtime);
            theme.with(runtime, |theme| {
                StyleProp::TextColor(variant.text_color(&theme.sidebar, chosen))
            })
        });
        if variant.has_pill() {
            cx.bind_checked(node, move |runtime| selected.get(runtime));
        }

        let mut layers = Vec::with_capacity(5);
        if variant.has_pill() {
            layers.push(cx.build(hover_wash(theme, move |runtime| {
                interaction.is_hovered(runtime) && !disabled.get(runtime) && !selected.get(runtime)
            })));
            layers.push(cx.build(selected_pill(theme, selected)));
        }
        if let Some(body) = self.icon {
            let size = variant.icon_size();
            layers.push(match variant {
                SidebarItemVariant::Brand => {
                    cx.build(fixed_icon(theme, body, size, LEADING_STROKE, |theme| {
                        theme.sidebar.brand_text
                    }))
                }
                _ => cx.build(leading_icon(theme, body, size, LEADING_STROKE, selected)),
            });
        }
        let label_gap = if self.icon.is_some() { ICON_GAP } else { 0.0 };
        layers.push(
            cx.build(
                text(self.label)
                    .truncate()
                    .grow(1.0)
                    .shrink(1.0)
                    .min_w(0.0)
                    .ml(label_gap),
            ),
        );
        if let Some(body) = self.trailing_icon {
            layers.push(cx.build(fixed_icon(
                theme,
                body,
                TRAILING_SIZE,
                TRAILING_STROKE,
                |theme| theme.sidebar.icon,
            )));
        }
        cx.adopt_children(node, layers);
    }
}
