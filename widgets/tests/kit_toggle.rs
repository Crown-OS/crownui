use crowntest::{Role, TestApp, role, text as text_query};
use view::{Listeners, Semantic, Styled, bind, component};
use widgets::{
    kit::{Theme, ToggleOptions, provide_theme, toggle},
    text, vstack,
};

fn settled_frame(app: &mut TestApp) -> String {
    app.settle();
    app.frame_snapshot()
}

fn lone_toggle(theme: Theme) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, theme);
        let on = cx.signal(false);
        vstack(toggle(on)).p(24.0)
    }))
}

#[test]
fn clicking_space_and_enter_each_flip_the_switch() {
    let mut app = TestApp::new(component(|cx| {
        let on = cx.signal(false);
        vstack((
            toggle(on).caption("Airplane mode"),
            text(bind(move |runtime| format!("On: {}", on.get(runtime)))),
        ))
    }));
    let switch = || role(Role::Switch).name("Airplane mode");
    assert_eq!(app.text_of(text_query("On:")), "On: false");
    assert_eq!(app.find(switch()).states.checked, Some(false));

    app.click(switch());
    assert_eq!(app.text_of(text_query("On:")), "On: true");
    assert_eq!(app.find(switch()).states.checked, Some(true));

    app.press_key("Space");
    assert_eq!(app.text_of(text_query("On:")), "On: false");
    app.press_key("Enter");
    assert_eq!(app.text_of(text_query("On:")), "On: true");
}

#[test]
fn a_disabled_switch_ignores_every_activation() {
    let mut app = TestApp::new(component(|cx| {
        let on = cx.signal(false);
        vstack((
            toggle(on).caption("Locked").disabled(true),
            text(bind(move |runtime| format!("On: {}", on.get(runtime)))),
        ))
    }));
    let center = app.find(role(Role::Switch)).bounds.center();
    app.pointer_down(center.x, center.y);
    app.pointer_up(center.x, center.y);
    assert_eq!(app.text_of(text_query("On:")), "On: false");
    let states = app.find(role(Role::Switch)).states;
    assert!(states.disabled && !states.focusable);
}

#[test]
fn the_signal_drives_the_switch_from_outside_too() {
    let mut app = TestApp::new(component(|cx| {
        let on = cx.signal(false);
        vstack((
            toggle(on).label("Wi-Fi"),
            widgets::kit::button("Turn on").on_click(move |cx, _| on.set(cx, true)),
        ))
    }));
    app.click(role(Role::Button).name("Turn on"));
    assert_eq!(
        app.find(role(Role::Switch).name("Wi-Fi")).states.checked,
        Some(true)
    );
}

#[test]
fn the_track_is_forty_eight_by_twenty_four_and_a_caption_follows_ten_pixels_on() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let on = cx.signal(false);
        vstack((toggle(on).label("Bare"), toggle(on).caption("Captioned"))).p(24.0)
    }));
    let bare = app.find(role(Role::Switch).name("Bare")).bounds;
    assert_eq!((bare.size.width, bare.size.height), (48.0, 24.0));
    let captioned = app.find(role(Role::Switch).name("Captioned")).bounds;
    assert!(captioned.size.width > 48.0 + 10.0);
    assert!(captioned.size.height >= 24.0);
    assert!(app.frame_snapshot().contains("layer at 58,"));
}

#[test]
fn the_off_track_paints_the_palette_gradient_with_the_knob_two_pixels_in() {
    let mut app = lone_toggle(Theme::dark());
    let frame = settled_frame(&mut app);
    assert!(
        frame.contains("fill 0,0 48x24 r12 gradient(#3f3f46ff@0,#2a2a2eff@1)"),
        "{frame}"
    );
    assert!(frame.contains("layer at 2,2"), "{frame}");
    assert!(frame.contains("fill 0,0 28x20 r10 #fafafaff"), "{frame}");
    assert!(
        frame.contains("shadow 0,0 28x20 r10 #00000024 blur4"),
        "{frame}"
    );
}

#[test]
fn turning_on_fades_in_the_accent_and_slides_the_knob_sixteen_pixels() {
    let mut app = lone_toggle(Theme::dark());
    let before = settled_frame(&mut app);
    assert!(!before.contains("#8f6dfbff"), "{before}");
    app.click(role(Role::Switch));
    let after = settled_frame(&mut app);
    assert!(
        after.contains("fill 0,0 48x24 r12 gradient(#8f6dfbff@0,#6d48e8ff@1)"),
        "{after}"
    );
    assert!(!after.contains("opacity"), "{after}");
    assert!(after.contains("layer at 18,2"), "{after}");
}

#[test]
fn the_knob_and_the_fade_spring_instead_of_snapping() {
    let mut app = lone_toggle(Theme::dark());
    app.click(role(Role::Switch));
    app.advance_ms(16);
    assert!(app.is_animating());
    let midway = app.frame_snapshot();
    assert!(!midway.contains("layer at 2,2"), "{midway}");
    assert!(!midway.contains("layer at 18,2"), "{midway}");
    assert!(midway.contains("opacity"), "{midway}");
    app.settle();
    assert!(!app.is_animating());
}

#[test]
fn the_track_follows_the_theme_and_the_accent() {
    let mut light = lone_toggle(Theme::light());
    assert!(
        settled_frame(&mut light).contains("gradient(#edebf6ff@0,#d9d6e8ff@1)"),
        "{}",
        light.frame_snapshot()
    );
    let mut app = TestApp::new(component(|cx| {
        let theme = provide_theme(cx, Theme::light());
        let on = cx.signal(true);
        vstack((
            toggle(on),
            widgets::kit::button("Dark").on_click(move |cx, _| theme.set(cx, Theme::dark())),
        ))
    }));
    assert!(app.frame_snapshot().contains("#edebf6ff"));
    app.click(role(Role::Button).name("Dark"));
    let frame = settled_frame(&mut app);
    assert!(
        frame.contains("#3f3f46ff") && !frame.contains("#edebf6ff"),
        "{frame}"
    );
}

#[test]
fn keyboard_focus_draws_no_halo_around_the_track() {
    let mut app = lone_toggle(Theme::dark());
    app.focus(role(Role::Switch));
    let frame = settled_frame(&mut app);
    assert!(!frame.contains("stroke3"), "{frame}");
}

#[test]
fn a_composed_frame_of_both_states_matches_its_snapshot() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::light());
        let off = cx.signal(false);
        let on = cx.signal(true);
        vstack((
            toggle(off).caption("Bluetooth"),
            toggle(on).caption("Wi-Fi"),
        ))
        .p(24.0)
        .gap(12.0)
    }));
    insta::assert_snapshot!(settled_frame(&mut app));
}
