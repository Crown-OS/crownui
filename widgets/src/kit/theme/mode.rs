/// Which of the two palettes a theme is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    /// Dark surfaces, light text. The CrownOS default.
    #[default]
    Dark,
    /// Light surfaces, dark text.
    Light,
}

impl ThemeMode {
    /// The mode `appearance.ron`'s `dark_mode` asks for.
    pub const fn from_dark_mode(dark_mode: bool) -> Self {
        if dark_mode { Self::Dark } else { Self::Light }
    }

    /// Whether this is [`Dark`](Self::Dark); for choosing the direction of a shadow or a tint.
    pub const fn is_dark(self) -> bool {
        matches!(self, Self::Dark)
    }
}
