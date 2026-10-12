use std::{fs, time::Duration};

use crownconfig::{AccentColor, Appearance, ThemeSettings, watch_theme_settings};

#[tokio::test]
async fn editing_either_file_updates_the_watched_settings() {
    let directory = std::env::temp_dir().join(format!("crownconfig-watch-{}", std::process::id()));
    fs::create_dir_all(&directory).expect("create the config directory");
    // SAFETY: this binary holds a single test, so no other thread reads the environment.
    unsafe { std::env::set_var("CROWN_CONFIG_DIR", &directory) };

    let mut watch = watch_theme_settings(ThemeSettings::default());
    assert_eq!(watch.current(), ThemeSettings::default());

    fs::write(
        directory.join("appearance.ron"),
        "(dark_mode: false, accent: Pink)",
    )
    .expect("write appearance");
    let after_appearance = tokio::time::timeout(Duration::from_secs(5), watch.changed())
        .await
        .expect("appearance edit is delivered");
    assert_eq!(
        after_appearance.appearance,
        Appearance {
            dark_mode: false,
            accent: AccentColor::Pink,
            ..Appearance::default()
        }
    );

    fs::write(
        directory.join("theme.ron"),
        r##"(dark: {"surface.bg": "#000000"})"##,
    )
    .expect("write theme");
    let after_theme = tokio::time::timeout(Duration::from_secs(5), watch.changed())
        .await
        .expect("theme edit is delivered");
    assert_eq!(after_theme.overrides.dark["surface.bg"], "#000000");
    assert_eq!(after_theme.appearance.accent, AccentColor::Pink);

    let _ = fs::remove_dir_all(&directory);
}
