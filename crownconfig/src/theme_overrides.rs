pub use crownos_config::schema::theme::{
    AccentGradient as AccentOverride, COLOR_TOKENS, Font, Radii, Theme as ThemeOverrides,
    is_color_token,
};

use crate::load_section;

/// Reads `theme.ron`, writing an empty one when it is missing.
pub fn load_theme_overrides() -> ThemeOverrides {
    load_section(ThemeOverrides::SECTION)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn a_sparse_file_parses_with_defaults_for_the_rest() {
        let overrides: ThemeOverrides =
            ron::from_str(r##"(dark: {"surface.bg": "#101010"})"##).expect("valid RON");
        assert_eq!(overrides.dark["surface.bg"], "#101010");
        assert!(overrides.light.is_empty() && overrides.accent.is_none());
        assert_eq!(overrides.radii, Radii::default());
    }

    #[test]
    fn overrides_round_trip() {
        let overrides = ThemeOverrides {
            accent: Some(AccentOverride {
                start: "#8f6dfb".into(),
                end: "#6d48e8".into(),
            }),
            light: BTreeMap::from([("text.body".to_owned(), "#202020".to_owned())]),
            ..ThemeOverrides::default()
        };
        let text = ron::to_string(&overrides).expect("serializes");
        assert_eq!(
            ron::from_str::<ThemeOverrides>(&text).expect("parses"),
            overrides
        );
    }
}
