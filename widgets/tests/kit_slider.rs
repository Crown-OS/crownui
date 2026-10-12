use crowntest::{Role, TestApp, role, text as text_query};
use geometry::{Point, Vector};
use input::{Modifiers, WheelDelta, WheelEvent};
use view::{Listeners, Semantic, Styled, bind, component};
use widgets::{
    kit::{SliderOptions, Theme, provide_theme, slider},
    text, vstack,
};
use windowing::WindowEvent;

const TRACK_LEFT: f32 = 24.0 + 13.0;
const TRACK_WIDTH: f32 = 454.0;

fn percent_slider(initial: f32) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, Theme::dark());
        let value = cx.signal(initial);
        vstack((
            slider(value).range(0.0, 100.0),
            text(bind(move |runtime| format!("Value {}", value.get(runtime)))),
        ))
        .p(24.0)
    }))
}

fn value_of(app: &mut TestApp) -> f32 {
    app.text_of(text_query("Value"))
        .trim_start_matches("Value ")
        .parse()
        .unwrap()
}

fn tap_at_progress(app: &mut TestApp, progress: f32) {
    let x = TRACK_LEFT + progress * TRACK_WIDTH;
    app.pointer_down(x, 39.0);
    app.pointer_up(x, 39.0);
}

fn settled_frame(app: &mut TestApp) -> String {
    app.settle();
    app.frame_snapshot()
}

#[test]
fn it_is_a_thirty_pixel_tall_slider_filling_its_column_up_to_four_hundred_eighty() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let value = cx.signal(0.5);
        vstack((
            slider(value).label("Wide"),
            vstack(slider(value).label("Narrow")).w(100.0),
            vstack(slider(value).label("Medium")).w(300.0),
        ))
        .p(24.0)
        .gap(8.0)
    }));
    let width_of = |app: &mut TestApp, name: &str| {
        let bounds = app.find(role(Role::Slider).name(name)).bounds;
        (bounds.size.width, bounds.size.height)
    };
    assert_eq!(width_of(&mut app, "Wide"), (480.0, 30.0));
    assert_eq!(width_of(&mut app, "Narrow"), (120.0, 30.0));
    assert_eq!(width_of(&mut app, "Medium"), (300.0, 30.0));
}

#[test]
fn the_thumb_sits_where_the_value_is_and_the_fill_gradient_spans_the_whole_track() {
    let mut app = percent_slider(25.0);
    let frame = settled_frame(&mut app);
    assert!(frame.contains("layer at 13,11"), "{frame}");
    assert!(
        frame.contains("fill 0,0 454x8 r4 gradient(#8f6dfbff@0,#6d48e8ff@1)"),
        "{frame}"
    );
    assert!(frame.contains("layer at 113.5,0"), "{frame}");
    assert!(frame.contains("layer at 113.5,5"), "{frame}");
    assert!(frame.contains("fill 0,0 26x20 r10 #fafafad9"), "{frame}");
    assert!(
        frame.contains("stroke1.75 -0.88,-0.88 27.75x21.75"),
        "{frame}"
    );
}

#[test]
fn clicking_the_track_jumps_the_value_under_the_pointer_and_clamps_at_the_ends() {
    let mut app = percent_slider(0.0);
    tap_at_progress(&mut app, 0.5);
    assert!((value_of(&mut app) - 50.0).abs() < 0.01);
    tap_at_progress(&mut app, 0.1);
    assert!((value_of(&mut app) - 10.0).abs() < 0.01);
    app.pointer_down(TRACK_LEFT - 12.0, 39.0);
    app.pointer_up(TRACK_LEFT - 12.0, 39.0);
    assert_eq!(value_of(&mut app), 0.0);
}

#[test]
fn dragging_follows_the_pointer_to_the_end_of_the_track() {
    let mut app = percent_slider(10.0);
    app.drag(role(Role::Slider), TRACK_LEFT + TRACK_WIDTH * 0.75, 39.0);
    assert!((value_of(&mut app) - 75.0).abs() < 0.01);
    app.drag(role(Role::Slider), 2000.0, 39.0);
    assert_eq!(value_of(&mut app), 100.0);
}

#[test]
fn arrows_home_end_and_page_keys_move_a_focused_slider() {
    let mut app = percent_slider(50.0);
    app.focus(role(Role::Slider));
    app.press_key("ArrowRight");
    assert_eq!(value_of(&mut app), 51.0);
    app.press_key("ArrowDown");
    assert_eq!(value_of(&mut app), 50.0);
    app.press_key("Shift+ArrowUp");
    assert_eq!(value_of(&mut app), 60.0);
    app.press_key("PageDown");
    assert_eq!(value_of(&mut app), 50.0);
    app.press_key("End");
    assert_eq!(value_of(&mut app), 100.0);
    app.press_key("ArrowRight");
    assert_eq!(value_of(&mut app), 100.0);
    app.press_key("Home");
    assert_eq!(value_of(&mut app), 0.0);
}

#[test]
fn enter_and_space_do_not_jump_the_thumb_to_the_middle() {
    let mut app = percent_slider(20.0);
    app.focus(role(Role::Slider));
    app.press_key("Enter");
    app.press_key("Space");
    assert_eq!(value_of(&mut app), 20.0);
}

#[test]
fn the_wheel_nudges_the_value_by_notches_and_touchpad_pixels() {
    let mut app = percent_slider(50.0);
    app.hover(role(Role::Slider));
    let wheel = |app: &mut TestApp, delta: WheelDelta| {
        app.inline()
            .handle(&WindowEvent::Wheel(WheelEvent {
                delta,
                position: Point::new(200.0, 39.0),
                modifiers: Modifiers::NONE,
            }))
            .unwrap();
    };
    wheel(&mut app, WheelDelta::Lines(Vector::new(0.0, 1.0)));
    assert_eq!(value_of(&mut app), 53.5);
    wheel(&mut app, WheelDelta::Pixels(Vector::new(-10.0, 0.0)));
    assert!((value_of(&mut app) - 50.5).abs() < 0.001);
}

#[test]
fn range_and_step_shape_the_values() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let value = cx.signal(0.0);
        vstack((
            slider(value).range(-10.0, 10.0).step(5.0),
            text(bind(move |runtime| format!("Value {}", value.get(runtime)))),
        ))
        .p(24.0)
    }));
    app.focus(role(Role::Slider));
    app.press_key("ArrowRight");
    assert_eq!(value_of(&mut app), 5.0);
    app.press_key("ArrowRight");
    app.press_key("ArrowRight");
    assert_eq!(value_of(&mut app), 10.0);
    tap_at_progress(&mut app, 0.3);
    assert_eq!(value_of(&mut app), -5.0);
}

#[test]
fn a_disabled_slider_ignores_the_pointer() {
    let mut app = TestApp::new(component(|cx| {
        let value = cx.signal(10.0);
        vstack((
            slider(value).range(0.0, 100.0).disabled(true),
            text(bind(move |runtime| format!("Value {}", value.get(runtime)))),
        ))
        .p(24.0)
    }));
    tap_at_progress(&mut app, 0.9);
    assert_eq!(value_of(&mut app), 10.0);
    let states = app.find(role(Role::Slider)).states;
    assert!(states.disabled && !states.focusable);
}

#[test]
fn assistive_technology_reads_a_slider_with_its_value() {
    let mut app = percent_slider(42.0);
    let node = app.find(role(Role::Slider));
    assert_eq!(node.value.as_deref(), Some("42"));
    assert!(node.states.focusable);
    app.focus(role(Role::Slider));
    app.press_key("ArrowRight");
    assert_eq!(app.find(role(Role::Slider)).value.as_deref(), Some("43"));
}

#[test]
fn the_thumb_and_fill_spring_to_a_new_value_together() {
    let mut app = percent_slider(0.0);
    assert!(!app.is_animating());
    app.focus(role(Role::Slider));
    app.press_key("End");
    app.advance_ms(16);
    assert!(app.is_animating());
    let midway = app.frame_snapshot();
    assert!(!midway.contains("layer at 454,0"), "{midway}");
    assert!(!midway.contains("layer at 0,0"), "{midway}");
    let settled = settled_frame(&mut app);
    assert!(settled.contains("layer at 454,0"), "{settled}");
    assert!(settled.contains("layer at 454,5"), "{settled}");
    assert!(!app.is_animating());
}

#[test]
fn focus_keeps_the_thumb_outline_and_adds_no_halo() {
    let mut app = percent_slider(50.0);
    app.focus(role(Role::Slider));
    let frame = settled_frame(&mut app);
    assert!(frame.contains("stroke1.75"), "{frame}");
    assert!(!frame.contains("stroke3"), "{frame}");
}

#[test]
fn the_thumb_has_no_bloom_in_either_palette() {
    for theme in [Theme::dark(), Theme::light()] {
        let mut app = TestApp::new(component(move |cx| {
            provide_theme(cx, theme);
            let value = cx.signal(0.5);
            vstack(slider(value)).p(24.0)
        }));
        let frame = settled_frame(&mut app);
        assert!(!frame.contains("shadow"), "{frame}");
    }
}

#[test]
fn the_palette_colors_the_track_and_thumb_and_follows_a_theme_switch() {
    let mut app = TestApp::new(component(|cx| {
        let theme = provide_theme(cx, Theme::light());
        let value = cx.signal(0.5);
        vstack((
            slider(value),
            widgets::kit::button("Dark").on_click(move |cx, _| theme.set(cx, Theme::dark())),
        ))
        .p(24.0)
    }));
    let light = app.frame_snapshot();
    assert!(light.contains("fill 0,0 454x8 #e7e5f1ff"), "{light}");
    assert!(light.contains("fill 0,0 26x20 r10 #ffffffd9"), "{light}");
    app.click(role(Role::Button).name("Dark"));
    let dark = settled_frame(&mut app);
    assert!(dark.contains("fill 0,0 454x8 #3a3a3dff"), "{dark}");
    assert!(dark.contains("fill 0,0 26x20 r10 #fafafad9"), "{dark}");
}

#[test]
fn a_composed_frame_matches_its_snapshot() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::light());
        let low = cx.signal(0.0);
        let mid = cx.signal(0.5);
        let full = cx.signal(1.0);
        vstack((slider(low), slider(mid), slider(full)))
            .p(24.0)
            .gap(8.0)
    }));
    insta::assert_snapshot!(settled_frame(&mut app));
}
