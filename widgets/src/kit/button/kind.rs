use geometry::{CornerRadii, Insets, Point, Rect};
use layouting::{Align, Dimension, FlexDirection, Justify};
use motion::Spring;
use reactive::Signal;
use semantics::Role;
use view::{Cx, Edges, Element, Kind, NodeId, Prop, SizeProperty, StyleProp, Styled, bind};
use windowing::CursorIcon;

use super::skin::{ButtonSize, ButtonVariant, HOVER_WASH, ICON_GAP, ICON_STROKE, PRESS_WASH, Skin};
use crate::{
    canvas::canvas,
    kit::{INTER_FAMILY, Theme, icon::tinted_icon, use_theme},
    stack::{hstack, vstack},
    text::text,
    util::{interaction::Interaction, spring, themed::themed},
};

/// The kind behind [`button`](super::button).
pub struct ButtonKind {
    pub(super) label: Prop<String>,
    pub(super) variant: ButtonVariant,
    pub(super) size: ButtonSize,
    pub(super) icon: Option<&'static str>,
    pub(super) min_width: f32,
}

impl ButtonKind {
    fn icon_only(&self) -> bool {
        self.size == ButtonSize::Icon && self.icon.is_some()
    }
}

fn skin_of(
    theme: Signal<Theme>,
    disabled: Signal<bool>,
    variant: ButtonVariant,
) -> impl Fn(&mut reactive::Runtime) -> Skin {
    move |runtime| {
        let is_disabled = disabled.get(runtime);
        theme.with(runtime, |theme| Skin::resolve(variant, theme, is_disabled))
    }
}

fn wash_layer(
    theme: Signal<Theme>,
    radius: f32,
    amount: impl FnMut(&mut reactive::Runtime) -> f32 + 'static,
    spring: Spring,
) -> Element<crate::stack::StackKind<()>> {
    vstack(())
        .absolute()
        .inset(0.0)
        .rounded(radius)
        .bg(themed(theme, |theme| theme.surface.hover))
        .opacity(bind(amount))
        .transition(spring)
}

impl Kind for ButtonKind {
    const NAME: &'static str = "kit_button";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let disabled = cx.observe_disabled(node);
        let interaction = Interaction::track(cx, node);
        let (variant, size, radius) = (self.variant, self.size, self.size.radius());
        let icon_only = self.icon_only();

        cx.set_role(node, Role::Button);
        for prop in [
            StyleProp::FlexDirection(FlexDirection::Row),
            StyleProp::AlignItems(Align::Center),
            StyleProp::Justify(Justify::Center),
            StyleProp::AlignSelf(if self.min_width.is_infinite() {
                Align::Stretch
            } else {
                Align::Start
            }),
            StyleProp::Position(layouting::Position::Relative),
            StyleProp::Size(SizeProperty::Height, Dimension::Length(size.height())),
            StyleProp::Size(
                SizeProperty::MinWidth,
                Dimension::Length(if icon_only {
                    size.height()
                } else if self.min_width.is_finite() {
                    self.min_width
                } else {
                    0.0
                }),
            ),
            StyleProp::Padding(Edges::Horizontal, size.horizontal_padding()),
            StyleProp::Radius(view::Corners::All, radius),
            StyleProp::Shrink(0.0),
            StyleProp::Cursor(CursorIcon::Pointer),
            StyleProp::TextSize(size.text_size()),
            StyleProp::FontFamily(INTER_FAMILY.to_owned()),
        ] {
            cx.set_default_style(node, prop);
        }
        let skin = skin_of(theme, disabled, variant);
        cx.bind_style(node, {
            let skin = skin_of(theme, disabled, variant);
            move |runtime| StyleProp::TextColor(skin(runtime).content)
        });

        let fill = canvas(move |runtime, bounds, list| {
            let skin = skin(runtime);
            let rect = Rect::new(Point::ZERO, bounds);
            list.fill(
                rect,
                CornerRadii::uniform(radius),
                skin.fill.vertical(0.0, bounds.height),
            );
            if let Some(border) = skin.border {
                list.stroke(
                    rect.inset(Insets::uniform(0.5)),
                    CornerRadii::uniform(radius - 0.5),
                    1.0,
                    border,
                );
            }
        })
        .absolute()
        .inset(0.0);

        let hover = wash_layer(
            theme,
            radius,
            move |runtime| {
                let washed = interaction.is_hovered(runtime)
                    && !interaction.is_pressed(runtime)
                    && !disabled.get(runtime);
                if washed { HOVER_WASH } else { 0.0 }
            },
            spring::snappy(),
        );
        let press = wash_layer(
            theme,
            radius,
            move |runtime| {
                if interaction.is_pressed(runtime) && !disabled.get(runtime) {
                    PRESS_WASH
                } else {
                    0.0
                }
            },
            spring::crisp(),
        );

        let glyph = self.icon.map(|body| {
            let skin = skin_of(theme, disabled, variant);
            tinted_icon(
                body,
                ICON_STROKE,
                bind(move |runtime| skin(runtime).content),
            )
            .size(size.icon_size())
        });
        let has_glyph = glyph.is_some();
        let label = text(self.label);
        let label = if icon_only {
            label.absolute().opacity(0.0)
        } else {
            label.ml(if has_glyph { ICON_GAP } else { 0.0 })
        };
        let content = hstack((glyph, label)).items_center();

        let layers = vec![
            cx.build(fill),
            cx.build(hover),
            cx.build(press),
            cx.build(content),
        ];
        cx.adopt_children(node, layers);
    }
}
