use crowntest::{Role, TestApp, role, text as text_query};
use view::{Listeners, Styled, component};
use widgets::{
    kit::{Theme, button, card, divider, provide_theme},
    text, vstack,
};

fn stage(theme: Theme, content: impl view::ViewSeq) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, theme);
        vstack(content).p(24.0).w(324.0)
    }))
}

#[test]
fn the_card_paints_the_raised_surface_with_a_one_pixel_border_at_radius_fourteen() {
    let app = stage(Theme::dark(), card(text("Network")));
    let frame = app.frame_snapshot();
    assert!(frame.contains("r14 #171718ff"), "{frame}");
    assert!(frame.contains("stroke1 0.5,0.5"), "{frame}");
    assert!(frame.contains("r13.5 #27272aff"), "{frame}");
}

#[test]
fn the_card_casts_no_shadow() {
    let app = stage(Theme::dark(), card(text("Network")));
    assert!(!app.frame_snapshot().contains("shadow"));
}

#[test]
fn the_content_sits_inside_the_border() {
    let mut app = stage(Theme::dark(), card(text("Network")));
    let label = app.find(text_query("Network")).bounds;
    assert_eq!((label.origin.x, label.origin.y), (25.0, 25.0));
}

#[test]
fn the_card_hugs_its_content_unless_told_to_stretch() {
    let mut hugging = stage(Theme::dark(), card(text("Network")));
    let mut stretched = stage(Theme::dark(), card(text("Network")).self_stretch());
    let hugging_layout = hugging.layout_snapshot();
    let stretched_layout = stretched.layout_snapshot();
    assert!(
        !hugging_layout.contains("kit_card @24,24 276x"),
        "{hugging_layout}"
    );
    assert!(
        stretched_layout.contains("kit_card @24,24 276x"),
        "{stretched_layout}"
    );
}

#[test]
fn rows_and_dividers_stack_inside_one_card() {
    let mut app = stage(
        Theme::dark(),
        card((text("Wi-Fi"), divider(), text("Bluetooth"))).self_stretch(),
    );
    let first = app.find(text_query("Wi-Fi")).bounds;
    let second = app.find(text_query("Bluetooth")).bounds;
    let separator = app.find(role(Role::Separator)).bounds;
    assert_eq!(separator.size.width, 274.0);
    assert!(first.max_y() <= separator.origin.y && separator.max_y() <= second.origin.y);
    insta::assert_snapshot!(app.snapshot());
}

#[test]
fn switching_the_theme_fades_the_surface() {
    let mut app = TestApp::new(component(|cx| {
        let theme = provide_theme(cx, Theme::light());
        vstack((
            card(text("Network")),
            button("Dark").on_click(move |cx, _| theme.set(cx, Theme::dark())),
        ))
        .p(24.0)
    }));
    assert!(app.frame_snapshot().contains("r14 #ffffffff"));
    app.click(role(Role::Button).name("Dark"));
    app.advance_ms(16);
    assert!(app.is_animating());
    app.settle();
    assert!(app.frame_snapshot().contains("r14 #171718ff"));
}
