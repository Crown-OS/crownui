use crowntest::{Role, TestApp, role, text as text_query};
use view::{Listeners, Semantic, Styled, bind, component};
use widgets::{
    kit::{Theme, button, provide_theme, radio_group},
    text, vstack,
};

const OPTIONS: [&str; 3] = ["Default", "Comfortable", "Compact"];
const ACCENT_RING: &str = "gradient(#8f6dfbff@0,#6d48e8ff@1)";

fn density_picker(initial: Option<usize>) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, Theme::dark());
        let selected = cx.signal(initial);
        vstack((
            radio_group(OPTIONS, selected).label("Density"),
            text(bind(move |runtime| {
                format!("Chosen {:?}", selected.get(runtime))
            })),
        ))
        .p(24.0)
    }))
}

fn row(name: &str) -> crowntest::Query {
    role(Role::RadioButton).name(name)
}

fn settled_frame(app: &mut TestApp) -> String {
    app.settle();
    app.frame_snapshot()
}

#[test]
fn clicking_a_row_selects_it_whether_the_circle_or_the_label_is_hit() {
    let mut app = density_picker(Some(0));
    app.click(row("Compact"));
    assert_eq!(app.text_of(text_query("Chosen")), "Chosen Some(2)");
    let bounds = app.find(row("Comfortable")).bounds;
    app.pointer_down(bounds.origin.x + 10.0, bounds.center().y);
    app.pointer_up(bounds.origin.x + 10.0, bounds.center().y);
    assert_eq!(app.text_of(text_query("Chosen")), "Chosen Some(1)");
    assert_eq!(app.find(row("Comfortable")).states.checked, Some(true));
    assert_eq!(app.find(row("Compact")).states.checked, Some(false));
}

#[test]
fn rows_are_thirty_apart_with_a_twenty_pixel_circle_and_the_label_thirty_five_in() {
    let mut app = density_picker(Some(0));
    let group = app.find(role(Role::Group).name("Density")).bounds;
    assert_eq!(group.size.height, 90.0);
    let rows: Vec<_> = OPTIONS
        .iter()
        .map(|name| app.find(row(name)).bounds)
        .collect();
    for (index, bounds) in rows.iter().enumerate() {
        assert_eq!(bounds.size.height, 30.0);
        assert_eq!(bounds.origin.y, group.origin.y + 30.0 * index as f32);
        assert_eq!(bounds.size.width, group.size.width);
    }
    let widest = app
        .find(role(Role::Text).name("Comfortable"))
        .bounds
        .size
        .width;
    assert!((group.size.width - (35.0 + widest)).abs() < 0.01);
    assert_eq!(
        app.find(role(Role::Text).name("Default")).bounds.origin.x,
        group.origin.x + 35.0
    );
    let frame = app.frame_snapshot();
    assert!(frame.contains("layer at 0,5"), "{frame}");
    assert!(frame.contains("fill 0,0 20x20 r10"), "{frame}");
}

#[test]
fn a_selected_circle_is_an_accent_ring_around_a_five_pixel_hole_and_the_rest_are_rims() {
    let mut app = density_picker(Some(0));
    let frame = settled_frame(&mut app);
    assert_eq!(frame.matches(ACCENT_RING).count(), 1, "{frame}");
    assert!(
        frame.contains("transform [0.5555556, 0.0, 0.0, 0.5555556, 5.0, 5.0]"),
        "{frame}"
    );
    assert!(frame.contains("fill 0,0 18x18 r9 #1d1d1dff"), "{frame}");
    assert_eq!(
        frame.matches("fill 0,0 18x18 r9 #1c1c1cff").count(),
        2,
        "{frame}"
    );
    assert_eq!(
        frame.matches("gradient(#3e3e3eff@0,#3e3e3eff@1)").count(),
        3,
        "{frame}"
    );
}

#[test]
fn nothing_selected_draws_three_rims() {
    let mut app = density_picker(None);
    let frame = settled_frame(&mut app);
    assert!(!frame.contains(ACCENT_RING), "{frame}");
    assert_eq!(
        frame.matches("fill 0,0 18x18 r9 #1c1c1cff").count(),
        3,
        "{frame}"
    );
}

#[test]
fn the_old_ring_thins_while_the_new_one_thickens_on_the_same_frames() {
    let mut app = density_picker(Some(0));
    assert!(!app.is_animating());
    app.click(row("Compact"));
    app.advance_ms(16);
    assert!(app.is_animating());
    let midway = app.frame_snapshot();
    assert_eq!(midway.matches("opacity").count(), 2, "{midway}");
    assert!(!midway.contains("transform [0.5555556"), "{midway}");
    let settled = settled_frame(&mut app);
    assert!(!app.is_animating());
    assert_eq!(settled.matches(ACCENT_RING).count(), 1, "{settled}");
    assert_eq!(
        settled.matches("transform [0.5555556").count(),
        1,
        "{settled}"
    );
    assert!(!settled.contains("opacity"), "{settled}");
}

#[test]
fn arrows_move_the_choice_wrapping_and_enter_does_nothing() {
    let mut app = density_picker(Some(1));
    app.focus(role(Role::Group).name("Density"));
    app.press_key("ArrowDown");
    assert_eq!(app.text_of(text_query("Chosen")), "Chosen Some(2)");
    app.press_key("ArrowDown");
    assert_eq!(app.text_of(text_query("Chosen")), "Chosen Some(0)");
    app.press_key("ArrowUp");
    assert_eq!(app.text_of(text_query("Chosen")), "Chosen Some(2)");
    app.press_key("Enter");
    app.press_key("Space");
    assert_eq!(app.text_of(text_query("Chosen")), "Chosen Some(2)");
}

#[test]
fn arrow_down_in_an_empty_group_picks_the_first_row() {
    let mut app = density_picker(None);
    app.focus(role(Role::Group).name("Density"));
    app.press_key("ArrowDown");
    assert_eq!(app.text_of(text_query("Chosen")), "Chosen Some(0)");
}

#[test]
fn the_group_is_one_tab_stop_without_a_focus_halo() {
    let mut app = density_picker(Some(1));
    assert!(app.find(role(Role::Group).name("Density")).states.focusable);
    assert!(!app.find(row("Default")).states.focusable);
    app.focus(role(Role::Group).name("Density"));
    let frame = settled_frame(&mut app);
    assert!(!frame.contains("stroke3"), "{frame}");
}

#[test]
fn a_disabled_group_ignores_input_and_mutes_every_color() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let selected = cx.signal(Some(0usize));
        vstack((
            radio_group(OPTIONS, selected).disabled(true),
            text(bind(move |runtime| {
                format!("Chosen {:?}", selected.get(runtime))
            })),
        ))
        .p(24.0)
    }));
    let bounds = app.find(row("Compact")).bounds;
    app.pointer_down(bounds.center().x, bounds.center().y);
    app.pointer_up(bounds.center().x, bounds.center().y);
    assert_eq!(app.text_of(text_query("Chosen")), "Chosen Some(0)");
    assert!(!app.find(role(Role::Group)).states.focusable);
    let frame = settled_frame(&mut app);
    assert!(
        frame.contains("gradient(#3e3e3e66@0,#3e3e3e66@1)"),
        "{frame}"
    );
    assert!(
        frame.contains("gradient(#8f6dfb66@0,#6d48e866@1)"),
        "{frame}"
    );
}

#[test]
fn assistive_technology_reads_the_selected_label_and_each_rows_state() {
    let mut app = density_picker(Some(2));
    let group = app.find(role(Role::Group).name("Density"));
    assert_eq!(group.value.as_deref(), Some("Compact"));
    assert_eq!(app.count(role(Role::RadioButton)), 3);
    assert_eq!(app.find(row("Compact")).states.checked, Some(true));
}

#[test]
fn the_palette_and_theme_switches_color_the_circles_and_labels() {
    let mut app = TestApp::new(component(|cx| {
        let theme = provide_theme(cx, Theme::light());
        let selected = cx.signal(Some(0usize));
        vstack((
            radio_group(OPTIONS, selected),
            button("Dark").on_click(move |cx, _| theme.set(cx, Theme::dark())),
        ))
        .p(24.0)
    }));
    let light = app.frame_snapshot();
    assert!(
        light.contains("gradient(#d9d6e8ff@0,#d9d6e8ff@1)"),
        "{light}"
    );
    assert!(light.contains("fill 0,0 18x18 r9 #ffffffff"), "{light}");
    app.click(role(Role::Button).name("Dark"));
    let dark = settled_frame(&mut app);
    assert!(dark.contains("gradient(#3e3e3eff@0,#3e3e3eff@1)"), "{dark}");
    assert!(!dark.contains("#d9d6e8ff"), "{dark}");
}

#[test]
fn a_composed_frame_matches_its_snapshot() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::light());
        let selected = cx.signal(Some(1usize));
        vstack(radio_group(OPTIONS, selected)).p(24.0)
    }));
    insta::assert_snapshot!(settled_frame(&mut app));
}
