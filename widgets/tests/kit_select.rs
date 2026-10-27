use crowntest::{Role, TestApp, role, text as text_query};
use geometry::Rect;
use reactive::Signal;
use view::{Cx, Listeners, Semantic, Styled, bind, component};
use widgets::{
    button,
    kit::{SelectOptions, Theme, provide_theme, section, select},
    spacer, text, vstack,
};

const OPTIONS: [&str; 5] = ["Next.js", "SvelteKit", "Nuxt.js", "Remix", "Astro"];
const PLACEHOLDER: &str = "Select a framework";
const FRAMEWORK: &str = "Framework";

struct Handles {
    selected: Signal<Option<usize>>,
}

fn framework_options() -> Vec<String> {
    OPTIONS.map(String::from).to_vec()
}

fn select_view(cx: &mut Cx, initial: Option<usize>) -> (Handles, impl view::View + use<>) {
    let options = cx.signal(framework_options());
    let selected = cx.signal(initial);
    let changed = cx.signal(String::new());
    let view = vstack((
        select(options, selected)
            .placeholder(PLACEHOLDER)
            .label(FRAMEWORK)
            .on_change(move |cx, event| changed.set(cx, event.value.clone())),
        button("Below"),
        button("Remix")
            .focusable(false)
            .on_click(move |cx, _| selected.set(cx, Some(3))),
        button("Shrink")
            .focusable(false)
            .on_click(move |cx, _| options.set(cx, vec!["Only one".into()])),
        button("Reorder").focusable(false).on_click(move |cx, _| {
            options.set(
                cx,
                ["1920x1200", "1920x1080", "1600x1200"]
                    .map(String::from)
                    .to_vec(),
            );
        }),
        button("Same")
            .focusable(false)
            .on_click(move |cx, _| options.set(cx, framework_options())),
        text(bind(move |runtime| {
            format!("Changed {}", changed.get(runtime))
        })),
    ))
    .p(24.0)
    .gap(12.0);
    (Handles { selected }, view)
}

fn select_app(theme: Theme, initial: Option<usize>) -> TestApp {
    TestApp::with_size(
        component(move |cx| {
            provide_theme(cx, theme);
            select_view(cx, initial).1
        }),
        480.0,
        520.0,
    )
}

fn combo() -> crowntest::Query {
    role(Role::ComboBox).name(FRAMEWORK)
}

fn open(app: &mut TestApp) {
    app.click(combo());
    app.settle();
}

fn option_bounds(app: &mut TestApp, label: &str) -> Rect {
    app.find(role(Role::ListItem).name(label)).bounds
}

fn shows(app: &mut TestApp, shown: &str) -> bool {
    app.exists(role(Role::Text).name(shown))
}

fn hex(color: geometry::Color) -> String {
    let channel = |value: f32| (value * 255.0).round() as u8;
    format!(
        "{:02x}{:02x}{:02x}{:02x}",
        channel(color.r),
        channel(color.g),
        channel(color.b),
        channel(color.a)
    )
}

fn changed(app: &mut TestApp) -> String {
    app.text_of(text_query("Changed"))
}

#[test]
fn the_trigger_is_a_252_by_34_pill_with_a_one_pixel_border_and_a_value_slot() {
    let theme = Theme::dark();
    let mut app = select_app(theme, None);
    let trigger = app.find(combo());
    assert_eq!(
        (trigger.bounds.size.width, trigger.bounds.size.height),
        (252.0, 34.0)
    );
    let frame = app.frame_snapshot();
    assert!(frame.contains("r16"), "{frame}");
    assert!(frame.contains("stroke1 0.5,0.5 251x33 r15.5"), "{frame}");
    assert!(
        frame.contains("text \"Select a framework\" at 0,0"),
        "{frame}"
    );
}

#[test]
fn the_placeholder_shows_until_something_is_selected_and_returns_when_it_is_cleared() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let (handles, view) = select_view(cx, None);
        vstack((
            view,
            button("Pick").on_click(move |cx, _| handles.selected.set(cx, Some(0))),
            button("Clear").on_click(move |cx, _| handles.selected.set(cx, None)),
        ))
    }));
    assert!(shows(&mut app, PLACEHOLDER));
    assert_eq!(app.find(combo()).value.as_deref(), Some(""));

    app.click(role(Role::Button).name("Pick"));
    app.settle();
    assert!(!shows(&mut app, PLACEHOLDER));
    assert!(shows(&mut app, "Next.js"));
    assert_eq!(app.find(combo()).value.as_deref(), Some("Next.js"));

    app.click(role(Role::Button).name("Clear"));
    app.settle();
    assert!(shows(&mut app, PLACEHOLDER));
}

#[test]
fn a_new_value_fades_in_after_the_old_one_fades_out_instead_of_snapping() {
    let mut app = select_app(Theme::dark(), Some(0));
    assert!(!app.frame_snapshot().contains("opacity"));
    app.click(role(Role::Button).name("Remix"));
    app.advance_ms(16);
    assert!(app.is_animating());
    let fading = app.frame_snapshot();
    assert!(
        fading.contains("text \"Next.js\""),
        "the old value is still fading out: {fading}"
    );
    assert!(fading.contains("text \"Remix\""), "{fading}");
    assert!(fading.contains("opacity"));
    app.settle();
    assert!(!shows(&mut app, "Next.js"));
    assert!(app.frame_snapshot().contains("text \"Remix\""));
    assert!(!app.frame_snapshot().contains("opacity"));
}

#[test]
fn a_value_the_select_is_built_with_is_simply_there() {
    let mut app = select_app(Theme::dark(), Some(1));
    assert!(!app.is_animating());
    assert!(shows(&mut app, "SvelteKit"));
    assert!(!app.frame_snapshot().contains("opacity"));
}

#[test]
fn the_panel_opens_five_pixels_below_the_trigger_as_wide_as_it_without_moving_siblings() {
    let mut app = select_app(Theme::dark(), None);
    let trigger = app.find(combo()).bounds;
    let sibling = app.find(role(Role::Button).name("Below")).bounds;
    assert!(!app.exists(role(Role::List)));

    open(&mut app);
    let panel = app.find(role(Role::List)).bounds;
    assert_eq!(panel.origin.x, trigger.origin.x);
    assert_eq!(panel.origin.y, trigger.max_y() + 5.0);
    assert_eq!(panel.size.width, 252.0);
    assert_eq!(panel.size.height, 5.0 * 2.0 + 5.0 * 30.0);
    assert_eq!(app.find(role(Role::Button).name("Below")).bounds, sibling);

    for (index, label) in OPTIONS.iter().enumerate() {
        let row = option_bounds(&mut app, label);
        assert_eq!(row.size.height, 30.0);
        assert_eq!(row.origin.x, panel.origin.x + 5.0);
        assert_eq!(row.size.width, 242.0);
        assert_eq!(row.origin.y, panel.origin.y + 5.0 + 30.0 * index as f32);
    }
}

#[test]
fn the_panel_springs_open_instead_of_appearing() {
    let mut app = select_app(Theme::dark(), None);
    app.click(combo());
    app.advance_ms(16);
    assert!(app.is_animating());
    let first = app.frame_snapshot();
    assert!(first.contains("opacity"), "{first}");
    app.advance_ms(48);
    assert_ne!(first, app.frame_snapshot());
    app.settle();
    assert!(!app.is_animating());
    assert!(!app.frame_snapshot().contains("opacity"));
}

#[test]
fn picking_an_option_selects_it_fires_change_and_closes_the_panel() {
    let mut app = select_app(Theme::dark(), None);
    open(&mut app);
    app.click(role(Role::ListItem).name("SvelteKit"));
    assert_eq!(changed(&mut app), "Changed SvelteKit");
    app.settle();
    assert!(!app.exists(role(Role::List)));
    assert_eq!(app.find(combo()).value.as_deref(), Some("SvelteKit"));
}

#[test]
fn a_press_with_the_pointer_picks_through_the_overlapping_content_beneath_it() {
    let mut app = select_app(Theme::dark(), None);
    open(&mut app);
    let row = option_bounds(&mut app, "Remix").center();
    app.pointer_down(row.x, row.y);
    assert!(
        app.exists(role(Role::List)),
        "the press must not blur the select"
    );
    app.pointer_up(row.x, row.y);
    assert_eq!(changed(&mut app), "Changed Remix");
}

#[test]
fn the_panel_escapes_a_clipping_ancestor_and_its_rows_stay_pickable() {
    let mut app = TestApp::with_size(
        component(|cx| {
            provide_theme(cx, Theme::dark());
            let options = cx.signal(framework_options());
            let selected = cx.signal(None);
            let changed = cx.signal(String::new());
            vstack((
                section(
                    "Clipped",
                    select(options, selected)
                        .label(FRAMEWORK)
                        .on_change(move |cx, event| changed.set(cx, event.value.clone())),
                ),
                text(bind(move |runtime| {
                    format!("Changed {}", changed.get(runtime))
                })),
            ))
            .p(24.0)
        }),
        480.0,
        520.0,
    );
    open(&mut app);
    let card = app.find(role(Role::Group).name("Clipped")).bounds;
    let row = option_bounds(&mut app, "Astro");
    assert!(row.min_y() > card.max_y(), "{row:?} sits below {card:?}");
    let frame = app.frame_snapshot();
    let panel_layer = frame.rfind("\nlayer").unwrap_or_default();
    assert!(frame[..panel_layer].contains("clip"), "{frame}");
    assert!(!frame[panel_layer..].contains("clip"), "{frame}");
    assert!(frame[panel_layer..].contains("text \"Astro\""), "{frame}");
    let center = row.center();
    app.pointer_down(center.x, center.y);
    app.pointer_up(center.x, center.y);
    assert_eq!(changed(&mut app), "Changed Astro");
}

#[test]
fn a_press_outside_the_panel_dismisses_it() {
    let mut app = select_app(Theme::dark(), None);
    open(&mut app);
    app.pointer_down(460.0, 500.0);
    app.pointer_up(460.0, 500.0);
    app.settle();
    assert!(!app.exists(role(Role::List)));
    assert_eq!(changed(&mut app), "Changed ");
}

#[test]
fn a_second_click_on_the_trigger_closes_the_panel_and_leaves_it_focused() {
    let mut app = select_app(Theme::dark(), None);
    open(&mut app);
    app.click(combo());
    app.settle();
    assert!(!app.exists(role(Role::List)));
    assert!(app.find(combo()).states.focused);
}

#[test]
fn the_hovered_row_gets_a_pill_that_travels_to_the_next_row_and_fades_out_on_leaving() {
    let theme = Theme::dark();
    let mut app = select_app(theme, None);
    open(&mut app);
    let hover_fill = format!("r16 #{}", hex(theme.popover.hover_bg));
    let hover_fill = hover_fill.as_str();
    assert!(!app.frame_snapshot().contains(hover_fill));

    let first = option_bounds(&mut app, "SvelteKit").center();
    app.pointer_move(first.x, first.y);
    app.settle();
    assert!(app.frame_snapshot().contains(hover_fill));
    let resting = app.frame_snapshot();

    let second = option_bounds(&mut app, "Remix").center();
    app.pointer_move(second.x, second.y);
    app.advance_ms(16);
    assert!(app.is_animating(), "the pill glides between rows");
    assert_ne!(app.frame_snapshot(), resting);
    app.settle();

    app.pointer_move(460.0, 500.0);
    app.advance_ms(16);
    assert!(app.is_animating(), "and fades rather than vanishing");
    app.settle();
    assert!(!app.frame_snapshot().contains(hover_fill));
}

#[test]
fn the_checkmark_marks_the_selected_row_and_glides_when_the_selection_moves_while_open() {
    let mut app = TestApp::with_size(
        component(|cx| {
            provide_theme(cx, Theme::dark());
            let (_, view) = select_view(cx, Some(1));
            view
        }),
        480.0,
        520.0,
    );
    open(&mut app);
    assert_eq!(
        app.find(role(Role::ListItem).name("SvelteKit"))
            .states
            .checked,
        Some(true)
    );
    assert_eq!(
        app.find(role(Role::ListItem).name("Remix")).states.checked,
        Some(false)
    );
    let before = app.frame_snapshot();
    assert!(before.contains("stroke1.8 path[3]"), "{before}");

    app.click(role(Role::Button).name("Remix"));
    app.advance_ms(16);
    assert!(
        app.is_animating(),
        "the checkmark travels instead of jumping"
    );
    app.settle();
    assert_eq!(
        app.find(role(Role::ListItem).name("Remix")).states.checked,
        Some(true)
    );
    assert_eq!(
        app.find(role(Role::ListItem).name("SvelteKit"))
            .states
            .checked,
        Some(false)
    );
    assert!(
        app.exists(role(Role::List)),
        "changing the selection from outside keeps the panel open"
    );
}

#[test]
fn the_panel_flips_above_a_trigger_with_no_room_below_and_stays_below_when_neither_fits() {
    let tall = |window_height: f32| {
        let mut app = TestApp::with_size(
            component(|cx| {
                provide_theme(cx, Theme::dark());
                let options = cx.signal(framework_options());
                let selected = cx.signal(None);
                vstack((spacer(), select(options, selected).label(FRAMEWORK)))
                    .p(24.0)
                    .h_full()
            }),
            400.0,
            window_height,
        );
        let trigger = app.find(combo()).bounds;
        open(&mut app);
        (trigger, app.find(role(Role::List)).bounds)
    };
    let (trigger, panel) = tall(400.0);
    assert_eq!(panel.max_y(), trigger.origin.y - 5.0);
    let (trigger, panel) = tall(100.0);
    assert_eq!(panel.origin.y, trigger.max_y() + 5.0);
}

#[test]
fn the_panel_hangs_off_a_trigger_that_is_not_at_the_window_origin() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let options = cx.signal(framework_options());
        let selected = cx.signal(None);
        vstack(select(options, selected).label(FRAMEWORK))
            .pl(40.0)
            .pt(60.0)
    }));
    let trigger = app.find(combo()).bounds;
    assert_eq!((trigger.origin.x, trigger.origin.y), (40.0, 60.0));
    open(&mut app);
    let panel = app.find(role(Role::List)).bounds;
    assert_eq!((panel.origin.x, panel.origin.y), (40.0, 99.0));
}

#[test]
fn a_new_option_list_rereads_the_selected_row_and_dismisses_an_open_panel() {
    let mut app = select_app(Theme::dark(), Some(1));
    assert!(shows(&mut app, "SvelteKit"));
    app.click(role(Role::Button).name("Reorder"));
    app.settle();
    assert!(
        shows(&mut app, "1920x1080"),
        "the index did not move, what it names did"
    );
    assert_eq!(app.find(combo()).value.as_deref(), Some("1920x1080"));

    open(&mut app);
    assert!(app.exists(role(Role::List)));
    app.click(role(Role::Button).name("Same"));
    app.settle();
    assert!(
        !app.exists(role(Role::List)),
        "rows of a list nobody has must not stay on screen"
    );
}

#[test]
fn an_unchanged_option_list_does_not_flicker_the_value() {
    let mut app = select_app(Theme::dark(), Some(0));
    app.click(role(Role::Button).name("Same"));
    assert!(!app.is_animating());
    assert!(!app.frame_snapshot().contains("opacity"));
}

#[test]
fn a_selection_that_no_longer_exists_falls_back_to_the_placeholder() {
    let mut app = select_app(Theme::dark(), Some(4));
    assert!(shows(&mut app, "Astro"));
    app.click(role(Role::Button).name("Shrink"));
    app.settle();
    assert!(shows(&mut app, PLACEHOLDER));
    assert_eq!(app.find(combo()).value.as_deref(), Some(""));
}

#[test]
fn keyboard_focus_sends_the_border_to_the_accent_gradient_without_a_halo() {
    let mut app = select_app(Theme::dark(), None);
    app.focus(combo());
    app.advance_ms(16);
    assert!(app.is_animating());
    app.settle();
    let focused = app.frame_snapshot();
    assert!(!focused.contains("stroke3"), "{focused}");
    assert!(focused.contains("gradient(#"), "{focused}");
}

#[test]
fn arrow_keys_open_the_panel_on_the_selected_row_and_enter_picks_the_highlighted_one() {
    let mut app = select_app(Theme::dark(), Some(1));
    app.focus(combo());
    app.press_key("ArrowDown");
    app.settle();
    assert!(app.exists(role(Role::List)));
    app.press_key("ArrowDown");
    app.press_key("ArrowDown");
    app.press_key("Enter");
    assert_eq!(changed(&mut app), "Changed Remix");
    app.settle();
    assert!(!app.exists(role(Role::List)));
}

#[test]
fn home_and_end_jump_the_highlight_and_escape_closes_without_changing_anything() {
    let mut app = select_app(Theme::dark(), Some(2));
    app.focus(combo());
    app.press_key("ArrowUp");
    app.press_key("End");
    app.press_key("Enter");
    assert_eq!(changed(&mut app), "Changed Astro");
    app.settle();

    app.press_key("ArrowDown");
    app.press_key("Home");
    app.press_key("Escape");
    app.settle();
    assert!(!app.exists(role(Role::List)));
    assert_eq!(changed(&mut app), "Changed Astro");
}

#[test]
fn typing_jumps_to_the_option_starting_with_the_letters_typed() {
    let mut app = select_app(Theme::dark(), None);
    app.focus(combo());
    app.press_key("r");
    assert_eq!(changed(&mut app), "Changed Remix");
    app.press_key("n");
    assert_eq!(changed(&mut app), "Changed Next.js");
    app.press_key("n");
    assert_eq!(changed(&mut app), "Changed Nuxt.js");
    app.press_key("z");
    assert_eq!(changed(&mut app), "Changed Nuxt.js");
}

#[test]
fn typing_while_open_moves_the_highlight_and_enter_picks_it() {
    let mut app = select_app(Theme::dark(), None);
    app.focus(combo());
    app.press_key("ArrowDown");
    app.press_key("a");
    app.press_key("Enter");
    assert_eq!(changed(&mut app), "Changed Astro");
}

#[test]
fn a_disabled_select_cannot_be_opened_and_reports_disabled() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let options = cx.signal(framework_options());
        let selected = cx.signal(None);
        vstack(select(options, selected).label(FRAMEWORK).disabled(true)).p(24.0)
    }));
    let trigger = app.find(combo());
    assert!(trigger.states.disabled);
    let id = trigger.id;
    assert!(app.ui().perform(id, &semantics::UiAction::Click).is_err());
    assert!(!app.exists(role(Role::List)));
}

#[test]
fn the_trigger_and_panel_follow_the_palette_when_the_theme_switches() {
    let mut app = TestApp::new(component(|cx| {
        let theme = provide_theme(cx, Theme::light());
        let options = cx.signal(framework_options());
        let selected = cx.signal(Some(0));
        vstack((
            select(options, selected).label(FRAMEWORK),
            button("Dark").on_click(move |cx, _| theme.set(cx, Theme::dark())),
        ))
        .p(24.0)
    }));
    let (light, dark) = (
        hex(Theme::light().popover.trigger_bg),
        hex(Theme::dark().popover.trigger_bg),
    );
    assert!(app.frame_snapshot().contains(&light));
    assert!(!app.frame_snapshot().contains(&dark));
    app.click(role(Role::Button).name("Dark"));
    app.settle();
    assert!(app.frame_snapshot().contains(&dark));
    assert!(!app.frame_snapshot().contains(&light));
}

#[test]
fn the_open_panel_paints_as_designed_in_both_palettes() {
    for (name, theme) in [("dark", Theme::dark()), ("light", Theme::light())] {
        let mut app = select_app(theme, Some(2));
        open(&mut app);
        let hover = option_bounds(&mut app, "Remix").center();
        app.pointer_move(hover.x, hover.y);
        app.settle();
        insta::with_settings!({ snapshot_suffix => name }, {
            insta::assert_snapshot!("open_select", app.frame_snapshot());
        });
    }
}

#[test]
fn the_new_value_rises_only_after_the_old_one_has_mostly_faded() {
    let mut app = select_app(Theme::dark(), Some(0));
    app.click(role(Role::Button).name("Remix"));
    let opacity_of = |frame: &str, text: &str| -> f32 {
        let lines: Vec<&str> = frame.lines().collect();
        let at = lines
            .iter()
            .position(|line| line.contains(&format!("text \"{text}\"")))
            .unwrap();
        lines[..at]
            .iter()
            .rev()
            .take(3)
            .find_map(|line| {
                line.split("opacity")
                    .nth(1)
                    .and_then(|rest| rest.trim().parse().ok())
            })
            .unwrap_or(1.0)
    };
    app.advance_ms(48);
    let early = app.frame_snapshot();
    assert!(opacity_of(&early, "Remix") < 0.1, "{early}");
    let outgoing = opacity_of(&early, "Next.js");
    assert!((0.1..0.7).contains(&outgoing), "{early}");
    app.advance_ms(144);
    let later = app.frame_snapshot();
    assert!(
        opacity_of(&later, "Remix") > opacity_of(&early, "Remix"),
        "{later}"
    );
}
