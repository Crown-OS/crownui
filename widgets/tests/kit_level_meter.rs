use crowntest::{Role, TestApp, role};
use view::{Listeners, Semantic, Styled, component};
use widgets::{
    kit::{LevelMeterOptions, Theme, button, level_meter, provide_theme},
    vstack,
};

const ACCENT: &str = "#8f6dfbff";

fn stage(content: impl view::ViewSeq) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, Theme::dark());
        vstack(content).p(24.0).gap(12.0)
    }))
}

fn fed_meter() -> TestApp {
    TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let peak = cx.signal(0.0_f32);
        vstack((
            level_meter(peak),
            button("Loud").on_click(move |cx, _| peak.set(cx, 1.0)),
            button("Quiet").on_click(move |cx, _| peak.set(cx, 0.1)),
        ))
        .p(24.0)
    }))
}

fn feed(app: &mut TestApp, name: &str, updates: usize) {
    for _ in 0..updates {
        app.click(role(Role::Button).name(name));
    }
}

#[test]
fn a_meter_is_a_two_hundred_forty_by_eight_bar_or_an_eight_by_one_twenty_column() {
    let mut app = stage((
        level_meter(0.5).label("Across"),
        level_meter(0.5).vertical().label("Upright"),
    ));
    for (name, size) in [("Across", (240.0, 8.0)), ("Upright", (8.0, 120.0))] {
        let bounds = app.find(role(Role::ProgressBar).name(name)).bounds;
        assert_eq!((bounds.size.width, bounds.size.height), size, "{name}");
    }
}

#[test]
fn the_fill_reads_on_a_sixty_decibel_scale_in_the_accent() {
    let app = stage(level_meter(0.1));
    let frame = app.frame_snapshot();
    assert!(
        frame.contains(&format!("fill 0,0 160x8 r4 {ACCENT}")),
        "{frame}"
    );
}

#[test]
fn a_vertical_meter_rises_from_the_bottom() {
    let app = stage(level_meter(0.1).vertical());
    let frame = app.frame_snapshot();
    assert!(
        frame.contains(&format!("fill 0,40 8x80 r4 {ACCENT}")),
        "{frame}"
    );
}

#[test]
fn the_color_warms_near_full_scale_and_turns_to_danger_when_clipping() {
    let theme = Theme::dark();
    for (peak, color) in [(0.6, theme.status.warning), (1.0, theme.status.danger)] {
        let [red, green, blue, alpha] = color.to_rgba8();
        let hex = format!("#{red:02x}{green:02x}{blue:02x}{alpha:02x}");
        let frame = stage(level_meter(peak)).frame_snapshot();
        assert!(frame.contains(&hex), "{peak}\n{frame}");
        assert!(!frame.contains(ACCENT), "{peak}\n{frame}");
    }
}

#[test]
fn rms_draws_solid_over_a_dimmed_peak() {
    let frame = stage(level_meter(0.1).rms(0.01)).frame_snapshot();
    assert!(frame.contains("fill 0,0 160x8 r4 #8f6dfb66"), "{frame}");
    assert!(
        frame.contains(&format!("fill 0,0 80x8 r4 {ACCENT}")),
        "{frame}"
    );
}

#[test]
fn silence_paints_nothing_and_reads_as_the_floor() {
    let mut app = stage(level_meter(0.0));
    assert!(!app.frame_snapshot().contains(ACCENT));
    let node = app.find(role(Role::ProgressBar));
    assert_eq!(node.value.as_deref(), Some("-60 dB"));
}

#[test]
fn assistive_technology_reads_the_peak_in_decibels() {
    let mut app = stage(level_meter(0.5));
    assert_eq!(
        app.find(role(Role::ProgressBar)).value.as_deref(),
        Some("-6 dB")
    );
}

#[test]
fn the_peak_hold_stays_for_sixty_updates_then_falls_back_to_the_level() {
    let mut app = fed_meter();
    feed(&mut app, "Loud", 1);
    feed(&mut app, "Quiet", 60);
    let held = app.frame_snapshot();
    assert!(held.contains("fill 237,0 3x8 r1 "), "{held}");
    feed(&mut app, "Quiet", 1);
    assert!(!app.frame_snapshot().contains("fill 237,0 3x8"));
    feed(&mut app, "Quiet", 40);
    let settled = app.frame_snapshot();
    assert!(!settled.contains("3x8 r1 "), "{settled}");
    assert!(!app.is_animating());
}

#[test]
fn meters_compose_in_a_column() {
    let mut app = stage((
        level_meter(0.02),
        level_meter(0.3).rms(0.1),
        level_meter(1.0),
        level_meter(0.4).vertical(),
    ));
    insta::assert_snapshot!(app.snapshot());
}
