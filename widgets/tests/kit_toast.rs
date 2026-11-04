use std::time::Duration;

use crowntest::{Role, TestApp, role};
use view::{Listeners, Styled, component};
use widgets::{
    kit::{
        Theme, Toast, ToastOptions, button, icons, provide_theme, toast, toast_host, use_toasts,
    },
    vstack,
};

fn window() -> TestApp {
    TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        toast_host(component(|cx| {
            let toasts = use_toasts(cx);
            vstack((
                button("Copy").on_click(move |cx, _| toasts.show(cx, "Copied 3 items")),
                button("Mount").on_click(move |cx, _| {
                    let long = Toast::new("Mounted USB Drive")
                        .icon(icons::HARD_DRIVE)
                        .duration(Duration::from_secs(6));
                    toasts.show_with(cx, long);
                }),
            ))
            .p(24.0)
            .items_start()
        }))
    }))
}

fn shows(app: &mut TestApp, message: &str) -> bool {
    app.exists(role(Role::Group).name(message))
}

#[test]
fn a_toast_appears_bottom_center_twenty_four_pixels_above_the_edge() {
    let mut app = window();
    app.click(role(Role::Button).name("Copy")).advance_ms(400);
    let pill = app.find(role(Role::Group).name("Copied 3 items")).bounds;
    assert!((pill.center().x - 400.0).abs() < 0.5, "{pill:?}");
    assert_eq!(pill.max_y(), 600.0 - 24.0);
    assert_eq!(pill.size.height, 36.0);
}

#[test]
fn a_toast_leaves_on_its_own_after_three_seconds() {
    let mut app = window();
    app.click(role(Role::Button).name("Copy")).advance_ms(2_800);
    assert!(shows(&mut app, "Copied 3 items"));
    app.advance_ms(400).settle();
    assert!(!shows(&mut app, "Copied 3 items"));
}

#[test]
fn toasts_stack_newest_last_and_keep_their_own_durations() {
    let mut app = window();
    app.click(role(Role::Button).name("Mount"));
    app.click(role(Role::Button).name("Copy")).advance_ms(400);
    let older = app.find(role(Role::Group).name("Mounted USB Drive")).bounds;
    let newer = app.find(role(Role::Group).name("Copied 3 items")).bounds;
    assert_eq!(newer.min_y(), older.max_y() + 8.0);
    app.advance_ms(3_000);
    assert!(!shows(&mut app, "Copied 3 items"));
    assert!(shows(&mut app, "Mounted USB Drive"));
    app.advance_ms(3_000).settle();
    assert!(!shows(&mut app, "Mounted USB Drive"));
}

#[test]
fn the_toast_layer_lets_presses_through_to_the_content() {
    let mut app = window();
    app.click(role(Role::Button).name("Copy"));
    let mount = app.find(role(Role::Button).name("Mount")).bounds.center();
    app.pointer_down(mount.x, mount.y)
        .pointer_up(mount.x, mount.y)
        .advance_ms(400);
    assert!(shows(&mut app, "Mounted USB Drive"));
}

#[test]
fn a_toast_with_an_icon_puts_it_eight_pixels_before_the_message() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        vstack(toast("Copied").leading_icon(icons::COPY))
            .p(24.0)
            .items_start()
    }));
    let layout = app.layout_snapshot();
    assert!(layout.contains("kit_toast > svg @38,34 16x16"), "{layout}");
    insta::assert_snapshot!(app.snapshot());
}
