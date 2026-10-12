use crowntest::{Role, TestApp, role, text as text_query};
use view::{Listeners, Semantic, Styled, bind, component};
use widgets::{
    kit::{Theme, ToggleGroupOptions, ToggleItem, button, icons, provide_theme, toggle_group},
    text, vstack,
};

fn modes() -> [ToggleItem; 3] {
    [
        ToggleItem::text("Light"),
        ToggleItem::text("Dark"),
        ToggleItem::text("Auto"),
    ]
}

fn mode_picker(initial: usize) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, Theme::dark());
        let selected = cx.signal(initial);
        vstack((
            toggle_group(modes(), selected).label("Appearance"),
            text(bind(move |runtime| {
                format!("Chosen {}", selected.get(runtime))
            })),
        ))
        .p(24.0)
    }))
}

fn chosen(app: &mut TestApp) -> usize {
    app.text_of(text_query("Chosen"))
        .trim_start_matches("Chosen ")
        .parse()
        .unwrap()
}

fn segment(name: &str) -> crowntest::Query {
    role(Role::RadioButton).name(name)
}

fn settled_frame(app: &mut TestApp) -> String {
    app.settle();
    app.frame_snapshot()
}

#[test]
fn clicking_a_segment_selects_it_and_reports_it_checked() {
    let mut app = mode_picker(0);
    assert_eq!(app.find(segment("Light")).states.checked, Some(true));
    app.click(segment("Auto"));
    assert_eq!(chosen(&mut app), 2);
    assert_eq!(app.find(segment("Auto")).states.checked, Some(true));
    assert_eq!(app.find(segment("Light")).states.checked, Some(false));
    app.click(segment("Auto"));
    assert_eq!(chosen(&mut app), 2);
}

#[test]
fn arrows_home_and_end_move_the_choice_and_stop_at_the_ends() {
    let mut app = mode_picker(1);
    app.focus(role(Role::Group).name("Appearance"));
    app.press_key("ArrowRight");
    assert_eq!(chosen(&mut app), 2);
    app.press_key("ArrowRight");
    assert_eq!(chosen(&mut app), 2);
    app.press_key("Home");
    assert_eq!(chosen(&mut app), 0);
    app.press_key("ArrowLeft");
    assert_eq!(chosen(&mut app), 0);
    app.press_key("End");
    assert_eq!(chosen(&mut app), 2);
    app.press_key("Enter");
    app.press_key("Space");
    assert_eq!(chosen(&mut app), 2);
}

#[test]
fn the_group_is_twenty_eight_tall_with_equal_segments_as_wide_as_the_widest_plus_padding() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let selected = cx.signal(0usize);
        vstack(
            toggle_group(
                [
                    ToggleItem::text("A"),
                    ToggleItem::text("A much longer label"),
                    ToggleItem::text("B"),
                ],
                selected,
            )
            .label("Uneven"),
        )
        .p(24.0)
    }));
    let group = app.find(role(Role::Group).name("Uneven")).bounds;
    assert_eq!(group.size.height, 28.0);
    let widths: Vec<f32> = ["A", "A much longer label", "B"]
        .iter()
        .map(|name| app.find(segment(name)).bounds.size.width)
        .collect();
    assert!((widths[0] - widths[1]).abs() < 0.01);
    assert!((widths[1] - widths[2]).abs() < 0.01);
    assert!(widths[1] > 24.0 + 80.0);
    assert!((group.size.width - (3.0 * widths[0] + 4.0)).abs() < 0.01);
    assert_eq!(app.find(segment("B")).bounds.size.height, 24.0);
}

#[test]
fn stretch_divides_the_width_the_parent_offers() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let selected = cx.signal(0usize);
        vstack(toggle_group(modes(), selected).stretch().label("Wide"))
            .p(24.0)
            .w(304.0)
    }));
    let group = app.find(role(Role::Group).name("Wide")).bounds;
    assert_eq!(group.size.width, 256.0);
    assert_eq!(app.find(segment("Dark")).bounds.size.width, 84.0);
}

#[test]
fn the_pill_rests_on_the_selection_with_the_palette_fill_and_border() {
    let mut app = mode_picker(1);
    let frame = settled_frame(&mut app);
    assert!(frame.contains("layer at 63,0"), "{frame}");
    assert!(frame.contains("fill 0,0 63x24 r12 #1c1c1cff"), "{frame}");
    assert!(
        frame.contains("stroke1 0.5,0.5 62x23 r11.5 #282828ff"),
        "{frame}"
    );
    assert!(frame.contains("fill 0,0 193x28 r14 #3a3a3dff"), "{frame}");
}

#[test]
fn choosing_springs_the_pill_across_and_cross_fades_the_labels() {
    let mut app = mode_picker(0);
    assert!(!app.is_animating());
    app.click(segment("Auto"));
    app.advance_ms(16);
    assert!(app.is_animating());
    let midway = app.frame_snapshot();
    assert!(
        !midway.contains("layer at 0,0") && !midway.contains("layer at 126,0"),
        "{midway}"
    );
    assert!(midway.contains("opacity"), "{midway}");
    let settled = settled_frame(&mut app);
    assert!(settled.contains("layer at 126,0"), "{settled}");
    assert!(!settled.contains("opacity"), "{settled}");
    assert!(!app.is_animating());
}

#[test]
fn a_hovered_unselected_segment_washes_at_once_and_the_selected_one_does_not() {
    let mut app = mode_picker(0);
    app.hover(segment("Dark"));
    assert!(!app.is_animating());
    let frame = app.frame_snapshot();
    assert!(frame.contains("fill 0,0 63x24 r12 #ffffff0a"), "{frame}");
    app.hover(segment("Light"));
    assert!(!app.frame_snapshot().contains("#ffffff0a"));
}

#[test]
fn a_disabled_group_ignores_input_and_dims_its_labels() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let selected = cx.signal(0usize);
        vstack((
            toggle_group(modes(), selected).disabled(true),
            text(bind(move |runtime| {
                format!("Chosen {}", selected.get(runtime))
            })),
        ))
        .p(24.0)
    }));
    let bounds = app.find(segment("Dark")).bounds;
    app.pointer_move(bounds.center().x, bounds.center().y);
    assert!(!app.frame_snapshot().contains("#ffffff0a"));
    app.pointer_down(bounds.center().x, bounds.center().y);
    app.pointer_up(bounds.center().x, bounds.center().y);
    assert_eq!(chosen(&mut app), 0);
    assert!(!app.find(role(Role::Group)).states.focusable);
}

#[test]
fn focus_draws_no_ring_around_the_pill() {
    let mut app = mode_picker(1);
    app.focus(role(Role::Group).name("Appearance"));
    let frame = settled_frame(&mut app);
    assert!(!frame.contains("stroke2"), "{frame}");
}

#[test]
fn assistive_technology_reads_the_selected_value_and_each_segments_name_once() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let selected = cx.signal(1usize);
        vstack(
            toggle_group(
                [
                    ToggleItem::icon(icons::SUN, "Light"),
                    ToggleItem::icon(icons::MOON, "Dark"),
                ],
                selected,
            )
            .label("Theme"),
        )
        .p(24.0)
    }));
    let group = app.find(role(Role::Group).name("Theme"));
    assert_eq!(group.value.as_deref(), Some("Dark"));
    assert!(group.states.focusable);
    assert_eq!(app.count(segment("Dark")), 1);
    assert_eq!(app.count(role(Role::Image)), 2);
}

#[test]
fn a_selection_past_the_last_segment_rests_on_the_last() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let selected = cx.signal(9usize);
        vstack(toggle_group(modes(), selected)).p(24.0)
    }));
    assert_eq!(app.find(segment("Auto")).states.checked, Some(true));
}

#[test]
fn the_track_and_labels_follow_the_theme() {
    let mut app = TestApp::new(component(|cx| {
        let theme = provide_theme(cx, Theme::light());
        let selected = cx.signal(0usize);
        vstack((
            toggle_group(modes(), selected),
            button("Dark").on_click(move |cx, _| theme.set(cx, Theme::dark())),
        ))
        .p(24.0)
    }));
    assert!(
        app.frame_snapshot()
            .contains("fill 0,0 193x28 r14 #e7e5f1ff")
    );
    app.click(role(Role::Button).name("Dark"));
    let dark = settled_frame(&mut app);
    assert!(dark.contains("fill 0,0 193x28 r14 #3a3a3dff"), "{dark}");
}

#[test]
fn a_composed_frame_matches_its_snapshot() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::light());
        let words = cx.signal(1usize);
        let icons = cx.signal(0usize);
        vstack((
            toggle_group(modes(), words),
            toggle_group(
                [
                    ToggleItem::icon(icons::SUN, "Light"),
                    ToggleItem::icon(icons::MOON, "Dark"),
                ],
                icons,
            ),
        ))
        .p(24.0)
        .gap(12.0)
    }));
    insta::assert_snapshot!(settled_frame(&mut app));
}
