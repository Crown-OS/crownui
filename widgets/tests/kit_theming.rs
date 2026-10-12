use crownconfig::AccentColor;
use crowntest::{Role, TestApp, role};
use geometry::Color;
use view::{Listeners, Prop, Styled, View, component};
use widgets::{
    hstack,
    kit::{
        ButtonOptions, ButtonVariant, SidebarColors, Theme, ThemeMode, accent_stops, button, icons,
        provide_theme, sidebar, sidebar_brand, sidebar_item, sidebar_separator, spinner,
        wizard_rail,
    },
    text, vstack,
};

const ALL_ACCENTS: [AccentColor; 5] = [
    AccentColor::Purple,
    AccentColor::Blue,
    AccentColor::Green,
    AccentColor::Orange,
    AccentColor::Pink,
];
const BOTH_MODES: [ThemeMode; 2] = [ThemeMode::Light, ThemeMode::Dark];

fn hex(color: Color) -> String {
    let [red, green, blue, alpha] = color.to_rgba8();
    format!("#{red:02x}{green:02x}{blue:02x}{alpha:02x}")
}

fn flat_gradient(color: Color) -> String {
    format!("gradient({0}@0,{0}@1)", hex(color))
}

fn gradient(start: Color, end: Color) -> String {
    format!("gradient({}@0,{}@1)", hex(start), hex(end))
}

fn frame_of(theme: Theme, view: impl View) -> String {
    let mut app = TestApp::new(component(move |cx| {
        provide_theme(cx, theme);
        view
    }));
    app.settle();
    app.frame_snapshot()
}

fn selected_row() -> impl View {
    sidebar(sidebar_item("Inbox", icons::INBOX, true), text("page"))
}

#[test]
fn a_primary_button_paints_the_configured_accent_gradient() {
    for accent in ALL_ACCENTS {
        let stops = accent_stops(accent);
        let theme = Theme::for_mode(ThemeMode::Dark, accent);
        let frame = frame_of(theme, button("Save").variant(ButtonVariant::Primary));
        assert!(
            frame.contains(&gradient(stops.start, stops.end)),
            "{accent:?}: {frame}"
        );
    }
}

#[test]
fn the_spinner_head_and_track_come_from_the_installed_palette() {
    for mode in BOTH_MODES {
        for accent in ALL_ACCENTS {
            let theme = Theme::for_mode(mode, accent);
            let mut app = TestApp::new(component(move |cx| {
                provide_theme(cx, theme);
                hstack(spinner()).p(8.0)
            }));
            app.advance_ms(400);
            let frame = app.frame_snapshot();
            assert!(
                frame.contains(&hex(theme.accent.end)),
                "{mode:?} {accent:?}"
            );
            assert!(
                frame.contains(&hex(theme.control.track.with_alpha_factor(0.55))),
                "{mode:?} {accent:?}"
            );
        }
    }
}

#[test]
fn a_selected_sidebar_row_paints_the_installed_mode() {
    for mode in BOTH_MODES {
        let theme = Theme::for_mode(mode, AccentColor::Purple);
        let frame = frame_of(theme, selected_row());
        assert!(
            frame.contains(&format!(
                "fill 0,0 235x35 r16 {}",
                hex(theme.sidebar.selected_bg)
            )),
            "{mode:?}: {frame}"
        );
        assert!(
            frame.contains(&format!(
                "fill 0,0 516x{} r16 {}",
                600 - 24,
                hex(theme.surface.bg)
            )),
            "{mode:?}: {frame}"
        );
    }
}

#[test]
fn the_two_modes_do_not_paint_the_same_thing() {
    let light = frame_of(Theme::light(), selected_row());
    let dark = frame_of(Theme::dark(), selected_row());
    assert_ne!(light, dark);
    let light = frame_of(Theme::light(), wizard_rail(["One", "Two"], Prop::Static(0)));
    let dark = frame_of(Theme::dark(), wizard_rail(["One", "Two"], Prop::Static(0)));
    assert_ne!(light, dark);
}

#[test]
fn every_sidebar_slot_is_read_from_the_palette_not_named_by_the_widget() {
    let marker = |red: u8| Color::from_rgba8(red, 0x31, 0x41, 0xff);
    let base = Theme::dark();
    let theme = Theme {
        sidebar: SidebarColors {
            separator: marker(2),
            hover_bg: marker(3),
            selected_bg: marker(4),
            selected_border: marker(5),
            icon: marker(6),
            selected_icon: marker(7),
            brand_text: marker(8),
            ..base.sidebar
        },
        ..base
    };
    let mut app = TestApp::new(component(move |cx| {
        provide_theme(cx, theme);
        sidebar(
            (
                sidebar_brand("Acme", icons::HOUSE),
                sidebar_item("Inbox", icons::INBOX, true),
                sidebar_item("Tasks", icons::SETTINGS, false),
                sidebar_separator(),
            ),
            text("page"),
        )
    }));
    app.hover(role(Role::MenuItem).name("Tasks"));
    app.settle();
    let frame = app.frame_snapshot();
    for slot in 2..=8 {
        assert!(frame.contains(&hex(marker(slot))), "slot {slot}: {frame}");
    }
}

#[test]
fn the_wizard_rail_follows_the_accent_and_the_track_color() {
    for accent in ALL_ACCENTS {
        let theme = Theme::for_mode(ThemeMode::Light, accent);
        let frame = frame_of(
            theme,
            vstack(wizard_rail(["A", "B", "C"], Prop::Static(1))).w(300.0),
        );
        assert!(frame.contains(&hex(theme.accent.end)), "{accent:?}");
        assert!(
            frame.contains(&hex(theme.control.track.with_alpha_factor(0.55))),
            "{accent:?}"
        );
    }
}

#[test]
fn the_destructive_fill_ignores_the_accent() {
    for mode in BOTH_MODES {
        let mut fills = Vec::new();
        for accent in ALL_ACCENTS {
            let theme = Theme::for_mode(mode, accent);
            let frame = frame_of(theme, button("Delete").variant(ButtonVariant::Destructive));
            assert!(
                frame.contains(&flat_gradient(theme.status.danger_bg)),
                "{mode:?} {accent:?}: {frame}"
            );
            fills.push(frame);
        }
        assert!(fills.windows(2).all(|pair| pair[0] == pair[1]));
    }
}

#[test]
fn switching_the_accent_at_runtime_repaints_accent_surfaces() {
    let mut app = TestApp::new(component(|cx| {
        let theme = provide_theme(cx, Theme::for_mode(ThemeMode::Dark, AccentColor::Purple));
        vstack((
            button("Save").variant(ButtonVariant::Primary),
            button("Green").on_click(move |cx, _| {
                theme.set(cx, Theme::for_mode(ThemeMode::Dark, AccentColor::Green));
            }),
        ))
    }));
    let purple = accent_stops(AccentColor::Purple);
    let green = accent_stops(AccentColor::Green);
    assert!(
        app.frame_snapshot()
            .contains(&gradient(purple.start, purple.end))
    );
    app.click(role(Role::Button).name("Green"));
    app.settle();
    let frame = app.frame_snapshot();
    assert!(frame.contains(&gradient(green.start, green.end)), "{frame}");
    assert!(
        !frame.contains(&gradient(purple.start, purple.end)),
        "{frame}"
    );
}
