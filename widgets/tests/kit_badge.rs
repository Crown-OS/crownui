use crowntest::{Role, TestApp, role, text as text_query};
use geometry::{Color, rgb};
use view::{Listeners, Styled, bind, component};
use widgets::{
    kit::{BadgeOptions, Theme, badge, button, icons, provide_theme, use_theme},
    vstack,
};

const BLUE: Color = rgb(0x2563eb);

fn badge_stage(content: impl view::ViewSeq) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, Theme::dark());
        vstack(content).p(24.0).items_start()
    }))
}

#[test]
fn a_badge_hugs_its_label_with_eight_and_three_pixels_of_padding() {
    let mut app = badge_stage(badge("Bluetooth", BLUE));
    let label = app.find(text_query("Bluetooth")).bounds;
    assert_eq!(label.origin.x, 24.0 + 8.0);
    assert_eq!(label.origin.y, 24.0 + 3.0);
    let layout = app.layout_snapshot();
    assert!(layout.contains("kit_badge @24,24 80.8x20.4"), "{layout}");
}

#[test]
fn the_label_is_twelve_pixel_text() {
    let mut app = badge_stage(badge("Beta", BLUE));
    let label = app.find(text_query("Beta")).bounds;
    assert!((label.size.height - 12.0 * 1.2).abs() < 0.01, "{label:?}");
}

#[test]
fn a_leading_icon_adds_twelve_pixels_and_a_four_pixel_gap() {
    let mut plain = badge_stage(badge("Bluetooth", BLUE));
    let mut with_icon = badge_stage(badge("Bluetooth", BLUE).leading_icon(icons::BLUETOOTH));
    let plain_label = plain.find(text_query("Bluetooth")).bounds;
    let icon_label = with_icon.find(text_query("Bluetooth")).bounds;
    assert_eq!(icon_label.origin.x - plain_label.origin.x, 12.0 + 4.0);
    let layout = with_icon.layout_snapshot();
    assert!(layout.contains("kit_badge > svg @32,"), "{layout}");
    assert!(layout.contains(" 12x12 "), "{layout}");
}

#[test]
fn the_icon_is_vertically_centred_in_the_pill() {
    let mut app = badge_stage(badge("Bluetooth", BLUE).leading_icon(icons::BLUETOOTH));
    let layout = app.layout_snapshot();
    let height = 3.0 * 2.0 + 14.4;
    let icon_top = 24.0 + (height - 12.0) / 2.0;
    assert!(
        layout.contains(&format!("kit_badge > svg @32,{icon_top} 12x12")),
        "{layout}"
    );
}

#[test]
fn the_wash_is_fourteen_percent_of_the_color_and_the_corner_radius_is_six() {
    let app = badge_stage(badge("Bluetooth", BLUE));
    let frame = app.frame_snapshot();
    assert!(frame.contains("r6 #2563eb24"), "{frame}");
}

#[test]
fn a_color_bound_to_the_theme_follows_it() {
    let mut app = TestApp::new(component(|cx| {
        let theme = provide_theme(cx, Theme::light());
        let tinted = use_theme(cx);
        vstack((
            badge(
                "Weak Security",
                bind(move |runtime| tinted.with(runtime, |theme| theme.status.warning)),
            ),
            button("Dark").on_click(move |cx, _| theme.set(cx, Theme::dark())),
        ))
        .p(24.0)
        .items_start()
    }));
    assert!(app.frame_snapshot().contains("#b4540924"));
    app.click(role(Role::Button).name("Dark"));
    app.advance_ms(16);
    assert!(app.is_animating());
    app.settle();
    let frame = app.frame_snapshot();
    assert!(
        frame.contains("#fbbf2424") && !frame.contains("#b4540924"),
        "{frame}"
    );
}

#[test]
fn a_badge_composes_beside_other_content() {
    let mut app = badge_stage((
        badge("Weak Security", rgb(0xfbbf24)),
        badge("Bluetooth", BLUE).leading_icon(icons::BLUETOOTH),
    ));
    insta::assert_snapshot!(app.snapshot());
}
