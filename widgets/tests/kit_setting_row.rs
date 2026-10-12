use crowntest::{Role, TestApp, role};
use view::{Semantic, Styled, component};
use widgets::{
    kit::{StatusOptions, Theme, Tone, provide_theme, setting_row, status},
    toggle,
};

fn stage(content: impl view::View) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, Theme::dark());
        widgets::vstack(content).p(24.0).w(408.0)
    }))
}

#[test]
fn the_row_carries_its_label_as_the_group_name() {
    let mut app = component_app();
    assert!(app.exists(role(Role::Group).name("Wi-Fi")));
}

fn component_app() -> TestApp {
    TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let enabled = cx.signal(true);
        widgets::vstack(setting_row("Wi-Fi", toggle(enabled).label("Wi-Fi")))
            .p(24.0)
            .w(408.0)
    }))
}

#[test]
fn the_label_sits_left_and_the_control_hugs_the_right_edge_inside_eighteen_by_fourteen_padding() {
    let mut app = component_app();
    let label = app.find(role(Role::Text).name("Wi-Fi")).bounds;
    let switch = app.find(role(Role::Switch).name("Wi-Fi")).bounds;
    assert_eq!(label.origin.x, 24.0 + 18.0);
    assert_eq!(switch.max_x(), 24.0 + 360.0 - 18.0);
    let row = app.find(role(Role::Group).name("Wi-Fi")).bounds;
    assert_eq!(row.size.height, 14.0 * 2.0 + switch.size.height);
}

#[test]
fn a_text_only_row_is_the_label_plus_vertical_padding() {
    let mut app = stage(setting_row(
        "Network",
        status("Connected").tone(Tone::Success),
    ));
    let row = app.find(role(Role::Group).name("Network")).bounds;
    let label = app.find(role(Role::Text).name("Network")).bounds;
    assert!((label.size.height - 14.0 * 1.2).abs() < 0.01);
    assert!((row.size.height - (28.0 + 16.8)).abs() < 0.01, "{row:?}");
}

#[test]
fn the_control_stays_interactive_inside_the_row() {
    let mut app = component_app();
    app.click(role(Role::Switch).name("Wi-Fi"));
    assert_eq!(
        app.find(role(Role::Switch).name("Wi-Fi")).states.checked,
        Some(false)
    );
}

#[test]
fn a_row_composes_with_a_status_control() {
    let mut app = stage(setting_row(
        "Network",
        status("Connected").tone(Tone::Success),
    ));
    insta::assert_snapshot!(app.snapshot());
}
