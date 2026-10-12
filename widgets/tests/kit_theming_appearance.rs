use std::{fs, path::PathBuf};

use crownconfig::AccentColor;
use crowntest::TestApp;
use geometry::Color;
use view::{Styled, View, component};
use widgets::{
    kit::{
        ButtonOptions, ButtonVariant, Theme, ThemeMode, accent_stops, button, icons,
        provide_configured_theme, sidebar, sidebar_item,
    },
    text, vstack,
};

struct ConfigDir(PathBuf);

impl ConfigDir {
    fn with_appearance(appearance: &str) -> Self {
        let directory =
            std::env::temp_dir().join(format!("crownui-theming-appearance-{}", std::process::id()));
        fs::create_dir_all(&directory).expect("create the config directory");
        fs::write(directory.join("appearance.ron"), appearance).expect("write appearance.ron");
        // SAFETY: this binary holds a single test, so no other thread reads the environment.
        unsafe { std::env::set_var("CROWN_CONFIG_DIR", &directory) };
        Self(directory)
    }
}

impl Drop for ConfigDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn hex(color: Color) -> String {
    let [red, green, blue, alpha] = color.to_rgba8();
    format!("#{red:02x}{green:02x}{blue:02x}{alpha:02x}")
}

fn screen() -> impl View {
    component(|cx| {
        provide_configured_theme(cx);
        vstack((
            button("Save").variant(ButtonVariant::Primary),
            sidebar(sidebar_item("Inbox", icons::INBOX, true), text("page")).h(200.0),
        ))
    })
}

#[test]
fn an_appearance_value_reaches_the_painted_widgets() {
    let _config = ConfigDir::with_appearance("(dark_mode: false, accent: Green)");
    let mut app = TestApp::new(screen());
    app.settle();
    let frame = app.frame_snapshot();
    let theme = Theme::for_mode(ThemeMode::Light, AccentColor::Green);
    let green = accent_stops(AccentColor::Green);
    assert!(
        frame.contains(&format!(
            "gradient({}@0,{}@1)",
            hex(green.start),
            hex(green.end)
        )),
        "{frame}"
    );
    assert!(frame.contains(&hex(theme.sidebar.selected_bg)), "{frame}");
    assert!(frame.contains(&hex(theme.surface.bg)), "{frame}");
}
