use std::{fs, path::PathBuf};

use crowntest::TestApp;
use geometry::rgb;
use view::component;
use widgets::{
    kit::{Theme, ThemeMode, accent_stops, provide_configured_theme},
    text,
};

struct ConfigDir(PathBuf);

impl ConfigDir {
    fn with_files(files: &[(&str, &str)]) -> std::io::Result<Self> {
        let directory = std::env::temp_dir().join(format!("crownui-theme-{}", std::process::id()));
        fs::create_dir_all(&directory)?;
        for (name, contents) in files {
            fs::write(directory.join(name), contents)?;
        }
        // SAFETY: this binary holds a single test, so no other thread reads the environment.
        unsafe { std::env::set_var("CROWN_CONFIG_DIR", &directory) };
        Ok(Self(directory))
    }
}

impl Drop for ConfigDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn configured_theme() -> Theme {
    let found = std::rc::Rc::new(std::cell::Cell::new(Theme::DEFAULT));
    let reported = found.clone();
    let _app = TestApp::new(component(move |cx| {
        let theme = provide_configured_theme(cx);
        reported.set(theme.get(cx));
        text("themed")
    }));
    found.get()
}

#[test]
fn the_theme_follows_appearance_ron_and_theme_ron() {
    let _config = ConfigDir::with_files(&[
        ("appearance.ron", "(dark_mode: false, accent: Orange)"),
        ("theme.ron", r##"(light: {"surface.bg": "#102030"})"##),
    ])
    .expect("write the config files");
    let theme = configured_theme();
    assert_eq!(theme.mode, ThemeMode::Light);
    assert_eq!(theme.accent, accent_stops(crownconfig::AccentColor::Orange));
    assert_eq!(theme.surface.bg, rgb(0x102030));
}
