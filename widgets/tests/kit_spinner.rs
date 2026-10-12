use crowntest::{Role, TestApp, role};
use view::{Styled, View, component};
use widgets::{
    hstack,
    kit::{Theme, provide_theme, spinner},
    text,
};

fn row() -> impl View {
    component(|cx| {
        provide_theme(cx, Theme::dark());
        hstack((spinner(), text("Looking for networks…")))
            .p(24.0)
            .gap(8.0)
            .items_center()
    })
}

#[test]
fn it_is_a_fifteen_pixel_progress_indicator() {
    let mut app = TestApp::new(row());
    let bounds = app.find(role(Role::ProgressBar)).bounds;
    assert_eq!((bounds.size.width, bounds.size.height), (15.0, 15.0));
}

#[test]
fn the_head_keeps_turning_and_the_spinner_never_settles() {
    let mut app = TestApp::new(row());
    app.advance_ms(300);
    let first = app.frame_snapshot();
    app.advance_ms(300);
    let second = app.frame_snapshot();
    assert_ne!(first, second);
    app.settle();
    assert!(
        app.is_animating(),
        "an indeterminate spinner must not settle"
    );
}

#[test]
fn the_ring_is_a_faint_track_under_an_accent_head() {
    let theme = Theme::dark();
    let mut app = TestApp::new(row());
    app.advance_ms(400);
    let frame = app.frame_snapshot();
    let track = format!(
        "stroke2.1 1.05,1.05 12.9x12.9 r6.45 {}",
        hex(theme.control.track.with_alpha_factor(0.55))
    );
    assert!(frame.contains(&track), "{frame}");
    assert!(
        frame.contains(&format!(
            "path[2] 7.5,7.5 6.45x6.45 {}",
            hex(theme.accent.end)
        )),
        "{frame}"
    );
}

#[test]
fn it_fades_in_from_nothing() {
    let mut app = TestApp::new(row());
    assert!(!app.frame_snapshot().contains("stroke"));
    app.advance_ms(16);
    assert!(app.frame_snapshot().contains("opacity0."));
    app.advance_ms(400);
    assert!(
        !app.frame_snapshot().contains("opacity0.9"),
        "{}",
        app.frame_snapshot()
    );
}

#[test]
fn a_larger_spinner_keeps_the_stroke_ratio() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        hstack(spinner().size(30.0)).p(8.0)
    }));
    app.advance_ms(400);
    assert!(
        app.frame_snapshot().contains("stroke4.2"),
        "{}",
        app.frame_snapshot()
    );
}

fn hex(color: geometry::Color) -> String {
    let [red, green, blue, alpha] = color.to_rgba8();
    format!("#{red:02x}{green:02x}{blue:02x}{alpha:02x}")
}
