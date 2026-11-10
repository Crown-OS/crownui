use crowntest::{Role, TestApp, role};
use view::{Styled, component};
use widgets::{
    kit::{Theme, button, provide_theme, tooltip},
    spacer, vstack,
};

fn toolbar() -> TestApp {
    TestApp::with_size(
        component(|cx| {
            provide_theme(cx, Theme::dark());
            vstack((
                tooltip("Go back", button("Back")),
                button("Elsewhere"),
                spacer(),
                tooltip("At the bottom", button("Low")),
            ))
            .p(24.0)
            .gap(8.0)
            .items_start()
            .h_full()
        }),
        400.0,
        300.0,
    )
}

fn tip(app: &mut TestApp, label: &str) -> bool {
    app.exists(role(Role::Tooltip).name(label))
}

#[test]
fn the_label_appears_only_after_the_pointer_rests_half_a_second() {
    let mut app = toolbar();
    app.hover(role(Role::Button).name("Back")).advance_ms(300);
    assert!(!tip(&mut app, "Go back"));
    app.advance_ms(300).settle();
    assert!(tip(&mut app, "Go back"));
}

#[test]
fn the_label_hangs_six_pixels_below_its_anchor() {
    let mut app = toolbar();
    app.hover(role(Role::Button).name("Back"))
        .advance_ms(600)
        .settle();
    let anchor = app.find(role(Role::Button).name("Back")).bounds;
    let bubble = app.find(role(Role::Tooltip)).bounds;
    assert_eq!(bubble.origin.y, anchor.max_y() + 6.0);
    assert!(
        (bubble.center().x - anchor.center().x).abs() < 0.5,
        "{bubble:?}"
    );
}

#[test]
fn near_the_window_bottom_the_label_flips_above() {
    let mut app = toolbar();
    app.hover(role(Role::Button).name("Low"))
        .advance_ms(600)
        .settle();
    let anchor = app.find(role(Role::Button).name("Low")).bounds;
    let bubble = app.find(role(Role::Tooltip)).bounds;
    assert!(
        (bubble.max_y() - (anchor.min_y() - 6.0)).abs() < 0.5,
        "{bubble:?} {anchor:?}"
    );
}

#[test]
fn leaving_before_the_delay_cancels_and_leaving_after_hides() {
    let mut app = toolbar();
    app.hover(role(Role::Button).name("Back")).advance_ms(200);
    app.hover(role(Role::Button).name("Elsewhere"))
        .advance_ms(600)
        .settle();
    assert!(!tip(&mut app, "Go back"));
    app.hover(role(Role::Button).name("Back"))
        .advance_ms(600)
        .settle();
    assert!(tip(&mut app, "Go back"));
    app.hover(role(Role::Button).name("Elsewhere")).settle();
    assert!(!tip(&mut app, "Go back"));
}

#[test]
fn pressing_the_anchor_hides_the_label() {
    let mut app = toolbar();
    app.hover(role(Role::Button).name("Back"))
        .advance_ms(600)
        .settle();
    let anchor = app.find(role(Role::Button).name("Back")).bounds.center();
    app.pointer_down(anchor.x, anchor.y).settle();
    assert!(!tip(&mut app, "Go back"));
}

fn showing_frames(app: &mut TestApp) -> Vec<String> {
    app.hover(role(Role::Button).name("Back")).advance_ms(500);
    (0..4)
        .map(|_| {
            let frame = app.advance_ms(16).frame_snapshot();
            let overlay = frame.rsplit_once("\nlayer at").map(|(_, overlay)| overlay);
            overlay.unwrap_or_default().to_owned()
        })
        .collect()
}

#[test]
fn the_first_label_inside_a_sprung_container_springs_like_every_later_one() {
    let mut app = TestApp::with_size(
        component(|cx| {
            provide_theme(cx, Theme::dark());
            vstack((tooltip("Go back", button("Back")), button("Elsewhere")))
                .transition(motion::Spring::smooth())
                .p(24.0)
                .gap(8.0)
                .items_start()
        }),
        400.0,
        300.0,
    );
    app.settle();
    let first = showing_frames(&mut app);
    app.hover(role(Role::Button).name("Elsewhere")).settle();
    assert_eq!(first, showing_frames(&mut app));
}
