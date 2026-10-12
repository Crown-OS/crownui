use std::sync::Arc;

use tokio::sync::watch;

use crate::{
    Appearance, ThemeOverrides, load_appearance, load_theme_overrides, watch::SectionWatch,
};

/// Everything the theme is derived from: `appearance.ron` and `theme.ron`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ThemeSettings {
    /// Mode, accent and animation profile.
    pub appearance: Appearance,
    /// Sparse palette edits.
    pub overrides: ThemeOverrides,
}

impl ThemeSettings {
    /// Reads both sections, writing the defaults of whichever is missing.
    pub fn load() -> Self {
        Self {
            appearance: load_appearance(),
            overrides: load_theme_overrides(),
        }
    }
}

/// Follows edits to either section, starting from `initial`.
pub fn watch_theme_settings(initial: ThemeSettings) -> SectionWatch<ThemeSettings> {
    let (sender, receiver) = watch::channel(initial);
    let sender = Arc::new(sender);
    let appearance_sender = Arc::clone(&sender);
    let appearance =
        crownos_config::subscribe_typed::<Appearance, _>(Appearance::SECTION, move |appearance| {
            appearance_sender.send_modify(|settings| settings.appearance = appearance);
        });
    let overrides = crownos_config::subscribe_typed::<ThemeOverrides, _>(
        ThemeOverrides::SECTION,
        move |overrides| {
            sender.send_modify(|settings| settings.overrides = overrides);
        },
    );
    SectionWatch::from_parts(receiver, vec![appearance, overrides])
}
