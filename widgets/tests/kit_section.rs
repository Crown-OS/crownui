use crowntest::{Role, TestApp, role};
use view::{Listeners, Semantic, Styled, component};
use widgets::{
    kit::{
        StatusOptions, Theme, Tone, button, divider, provide_theme, section, setting_row, status,
    },
    toggle, vstack,
};

fn connectivity() -> TestApp {
    TestApp::new(component(|cx| {
        let theme = provide_theme(cx, Theme::dark());
        let bluetooth = cx.signal(false);
        vstack((
            section(
                "Connectivity",
                (
                    setting_row("Bluetooth", toggle(bluetooth).label("Bluetooth")),
                    divider(),
                    setting_row("Network", status("Connected").tone(Tone::Success)),
                ),
            ),
            button("Light").on_click(move |cx, _| theme.set(cx, Theme::light())),
        ))
        .p(24.0)
        .w(408.0)
    }))
}

#[test]
fn the_section_is_a_group_named_by_its_title_with_a_heading() {
    let mut app = connectivity();
    assert!(app.exists(role(Role::Group).name("Connectivity")));
    assert!(app.exists(role(Role::Heading).name("Connectivity")));
}

#[test]
fn the_heading_is_twelve_pixel_text_inset_four_pixels_and_the_rows_sit_eight_below() {
    let mut app = connectivity();
    let heading = app.find(role(Role::Heading)).bounds;
    assert!((heading.size.height - 12.0 * 1.2).abs() < 0.01);
    let layout = app.layout_snapshot();
    assert!(
        layout.contains("kit_section > kit_card[1] @24,46.4 360x"),
        "{layout}"
    );
}

#[test]
fn the_rows_share_one_card_surface() {
    let app = connectivity();
    let frame = app.frame_snapshot();
    assert!(frame.contains("r14 #171718ff"), "{frame}");
    assert!(frame.contains("r13.5 #27272aff"), "{frame}");
    assert_eq!(frame.matches("r14 #171718ff").count(), 1, "{frame}");
}

#[test]
fn the_rows_are_clipped_to_the_surface() {
    let app = connectivity();
    assert!(app.frame_snapshot().contains("clip"));
}

#[test]
fn the_rows_inside_stay_interactive_and_theme_aware() {
    let mut app = connectivity();
    app.click(role(Role::Switch).name("Bluetooth"));
    assert_eq!(
        app.find(role(Role::Switch).name("Bluetooth"))
            .states
            .checked,
        Some(true)
    );
    app.click(role(Role::Button).name("Light"));
    app.advance_ms(16);
    assert!(app.is_animating());
    app.settle();
    assert!(app.frame_snapshot().contains("r14 #ffffffff"));
}

#[test]
fn a_section_composes_its_rows_under_the_heading() {
    let mut app = connectivity();
    app.settle();
    insta::assert_snapshot!(app.snapshot());
}
