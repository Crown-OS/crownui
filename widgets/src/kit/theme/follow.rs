use crownconfig::{SectionWatch, ThemeSettings, watch_theme_settings};
use reactive::Signal;
use view::Cx;

use super::{definition::Theme, environment::provide_theme};

/// Provides the theme `appearance.ron` and `theme.ron` describe to the current scope, and keeps it
/// following edits to either file while the calling component is mounted.
pub fn provide_configured_theme(cx: &mut Cx) -> Signal<Theme> {
    let settings = ThemeSettings::load();
    let theme = provide_theme(cx, Theme::from_settings(&settings));
    follow_config(cx, theme, watch_theme_settings(settings));
    theme
}

fn follow_config(cx: &mut Cx, theme: Signal<Theme>, mut watch: SectionWatch<ThemeSettings>) {
    let next_change = async move {
        let settings = watch.changed().await;
        (watch, settings)
    };
    cx.spawn(next_change, move |cx, (watch, settings)| {
        theme.set(cx, Theme::from_settings(&settings));
        follow_config(cx, theme, watch);
    });
}
