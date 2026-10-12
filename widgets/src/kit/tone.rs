use geometry::Color;

use super::Theme;

/// What a status line or callout is saying, in the four registers the palette provides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tone {
    /// It worked: "Connected", "Saved".
    Success,
    /// It works, but: "Weak Security".
    Warning,
    /// It failed: "Couldn't join network".
    Danger,
    /// Nothing in particular; the quiet default.
    #[default]
    Neutral,
}

impl Tone {
    /// The tone's own color: a status dot's fill, a callout's wash, rule and glyph.
    pub fn color(self, theme: &Theme) -> Color {
        let status = &theme.status;
        match self {
            Self::Success => status.success,
            Self::Warning => status.warning,
            Self::Danger => status.danger,
            Self::Neutral => status.neutral,
        }
    }

    /// The label's color: neutral recedes to muted text, every other tone letters in its own color.
    pub fn text_color(self, theme: &Theme) -> Color {
        match self {
            Self::Neutral => theme.text.muted,
            tone => tone.color(theme),
        }
    }
}
