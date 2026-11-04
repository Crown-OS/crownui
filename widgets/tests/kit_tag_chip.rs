use crowntest::{Role, TestApp, role, text as text_query};
use geometry::rgb;
use view::{Styled, component};
use widgets::{
    hstack,
    kit::{Theme, provide_theme, tag_chip},
};

fn chips(theme: Theme) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, theme);
        hstack((
            tag_chip("Red", rgb(0xe5484d)),
            tag_chip("Work", rgb(0x3b82f6)),
        ))
        .p(24.0)
        .gap(6.0)
        .items_start()
    }))
}

#[test]
fn a_chip_is_a_22_pixel_pill_with_eight_pixels_of_padding() {
    let mut app = chips(Theme::dark());
    let layout = app.layout_snapshot();
    assert!(layout.contains("kit_tag_chip @24,24 "), "{layout}");
    assert!(layout.contains("x22 "), "{layout}");
    let label = app.find(text_query("Red")).bounds;
    assert_eq!(label.origin.x, 24.0 + 8.0 + 6.0 + 6.0);
    assert!((label.size.height - 11.0 * 1.2).abs() < 0.01, "{label:?}");
}

#[test]
fn the_dot_is_six_pixels_in_the_tag_color_on_the_field_background() {
    let app = chips(Theme::dark());
    let frame = app.frame_snapshot();
    assert!(frame.contains("r3 #e5484d"), "{frame}");
    assert!(frame.contains("r11 #1c1c1c"), "{frame}");
    assert!(frame.contains("#282828"), "{frame}");
}

#[test]
fn chips_follow_the_theme_and_snapshot() {
    let mut app = chips(Theme::light());
    assert_eq!(app.count(role(Role::Text)), 2);
    insta::assert_snapshot!(app.snapshot());
}
