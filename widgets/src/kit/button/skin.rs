use geometry::{Color, TRANSPARENT};

use crate::{
    kit::{GradientStops, TEXT_SIZE, Theme},
    util::color::scale_alpha,
};

pub(super) const ICON_GAP: f32 = 6.0;
pub(super) const ICON_STROKE: f32 = 1.8;
pub(super) const HOVER_WASH: f32 = 0.7;
pub(super) const PRESS_WASH: f32 = 1.0;
const ICON_RADIUS: f32 = 8.0;
const TINT_LIGHT: f32 = 0.18;
const TINT_DARK: f32 = 0.26;
const DISABLED_FILL: f32 = 0.4;
const DISABLED_BORDER: f32 = 0.6;

/// Which slot of the palette fills a button; geometry is [`ButtonSize`]'s business.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    /// The accent gradient, lettered in `text.on_accent`. One per surface.
    Primary,
    /// A bordered chip: the everyday button.
    #[default]
    Secondary,
    /// No fill until hovered, for toolbars and tertiary actions.
    Ghost,
    /// A red fill for actions that destroy something.
    Destructive,
    /// A wash of the accent lettered in the accent itself.
    Tinted,
}

/// A button's geometry: height, padding, corner and text size, as one choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonSize {
    /// 28px tall.
    Small,
    /// 34px tall, a text field's height.
    #[default]
    Medium,
    /// A 28px rounded square showing only its icon.
    Icon,
}

impl ButtonSize {
    /// The button's height in logical pixels.
    pub const fn height(self) -> f32 {
        match self {
            Self::Small | Self::Icon => 28.0,
            Self::Medium => 34.0,
        }
    }

    pub(super) const fn horizontal_padding(self) -> f32 {
        match self {
            Self::Small => 14.0,
            Self::Medium => 20.0,
            Self::Icon => (Self::Icon.height() - Self::Icon.icon_size()) / 2.0,
        }
    }

    pub(super) const fn radius(self) -> f32 {
        match self {
            Self::Icon => ICON_RADIUS,
            Self::Small | Self::Medium => self.height() / 2.0,
        }
    }

    pub(super) const fn text_size(self) -> f32 {
        match self {
            Self::Small | Self::Icon => TEXT_SIZE - 2.0,
            Self::Medium => TEXT_SIZE,
        }
    }

    pub(super) const fn icon_size(self) -> f32 {
        match self {
            Self::Small => 13.0,
            Self::Medium | Self::Icon => 15.0,
        }
    }
}

/// The colors one variant resolves to.
pub(super) struct Skin {
    pub(super) fill: GradientStops,
    pub(super) border: Option<Color>,
    pub(super) content: Color,
}

impl Skin {
    pub(super) fn resolve(variant: ButtonVariant, theme: &Theme, disabled: bool) -> Self {
        let skin = match variant {
            ButtonVariant::Primary => Self {
                fill: theme.accent,
                border: None,
                content: theme.text.on_accent,
            },
            ButtonVariant::Secondary => Self {
                fill: GradientStops::flat(theme.popover.trigger_bg),
                border: Some(theme.popover.border),
                content: theme.text.body,
            },
            ButtonVariant::Ghost => Self {
                fill: GradientStops::flat(TRANSPARENT),
                border: None,
                content: theme.text.body,
            },
            ButtonVariant::Tinted => {
                let (strength, content) = if theme.mode.is_dark() {
                    (TINT_DARK, theme.accent.start)
                } else {
                    (TINT_LIGHT, theme.accent.end)
                };
                Self {
                    fill: theme.accent.fade(strength),
                    border: None,
                    content,
                }
            }
            ButtonVariant::Destructive => Self {
                fill: GradientStops::flat(theme.status.danger_bg),
                border: None,
                content: theme.status.on_danger,
            },
        };
        if !disabled {
            return skin;
        }
        Self {
            fill: skin.fill.fade(DISABLED_FILL),
            border: skin.border.map(|color| scale_alpha(color, DISABLED_BORDER)),
            content: theme.text.disabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_buttons_drop_their_fill_and_letter_in_disabled_text() {
        let theme = Theme::dark();
        let enabled = Skin::resolve(ButtonVariant::Primary, &theme, false);
        let disabled = Skin::resolve(ButtonVariant::Primary, &theme, true);
        assert!(disabled.fill.start.a < enabled.fill.start.a);
        assert_eq!(disabled.content, theme.text.disabled);
    }

    #[test]
    fn only_the_secondary_variant_has_a_border() {
        let theme = Theme::light();
        for variant in [
            ButtonVariant::Primary,
            ButtonVariant::Ghost,
            ButtonVariant::Destructive,
            ButtonVariant::Tinted,
        ] {
            assert!(Skin::resolve(variant, &theme, false).border.is_none());
        }
        assert!(
            Skin::resolve(ButtonVariant::Secondary, &theme, false)
                .border
                .is_some()
        );
    }

    #[test]
    fn tint_strength_follows_the_mode() {
        let light = Skin::resolve(ButtonVariant::Tinted, &Theme::light(), false);
        let dark = Skin::resolve(ButtonVariant::Tinted, &Theme::dark(), false);
        assert!(dark.fill.end.a > light.fill.end.a);
    }
}
