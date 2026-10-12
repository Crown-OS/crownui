use crowntest::{Role, TestApp, role, text as text_query};
use view::{Listeners, Styled, component};
use widgets::{
    kit::{Theme, button, card, provide_theme, soft_shadow},
    text, vstack,
};

fn stage(theme: Theme, content: impl view::ViewSeq) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, theme);
        vstack(content).p(24.0).items_start()
    }))
}

#[test]
fn the_shadow_is_a_ten_sigma_blur_three_pixels_below_the_shape() {
    let app = stage(Theme::dark(), soft_shadow(text("Network"), 14.0));
    let frame = app.frame_snapshot();
    let shadow = frame
        .lines()
        .find(|line| line.contains("shadow"))
        .unwrap_or_default();
    assert!(
        shadow.contains("r14") && shadow.contains("blur20") && shadow.contains("0,3"),
        "{frame}"
    );
}

#[test]
fn the_shadow_color_is_the_palettes_and_follows_the_mode() {
    let dark = stage(Theme::dark(), soft_shadow(text("Network"), 14.0));
    assert!(dark.frame_snapshot().contains("#00000059"));
    let light = stage(Theme::light(), soft_shadow(text("Network"), 14.0));
    assert!(light.frame_snapshot().contains("#14141f0d"));
}

#[test]
fn the_shadow_takes_the_size_of_its_child_and_adds_no_chrome_of_its_own() {
    let mut app = stage(Theme::dark(), soft_shadow(text("Network"), 14.0));
    let label = app.find(text_query("Network")).bounds;
    assert_eq!((label.origin.x, label.origin.y), (24.0, 24.0));
    let layout = app.layout_snapshot();
    assert!(
        layout.contains("kit_soft_shadow @24,24 58.8x16.8"),
        "{layout}"
    );
    assert!(!app.frame_snapshot().contains("fill"));
}

#[test]
fn a_card_can_wear_the_shadow_the_original_commented_out() {
    let app = stage(Theme::dark(), soft_shadow(card(text("Network")), 14.0));
    insta::assert_snapshot!(app.frame_snapshot());
}

#[test]
fn switching_the_theme_fades_the_shadow() {
    let mut app = TestApp::new(component(|cx| {
        let theme = provide_theme(cx, Theme::dark());
        vstack((
            soft_shadow(text("Network"), 14.0),
            button("Light").on_click(move |cx, _| theme.set(cx, Theme::light())),
        ))
        .p(24.0)
    }));
    app.click(role(Role::Button).name("Light"));
    app.advance_ms(16);
    assert!(app.is_animating());
    app.settle();
    assert!(app.frame_snapshot().contains("#14141f0d"));
}
