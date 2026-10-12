use crowntest::{Role, TestApp, role, text as text_query};
use reactive::Signal;
use view::{Cx, Element, Listeners, Semantic, Styled, bind, component};
use widgets::{
    kit::{
        ClearButton, FieldShape, SearchKind, SearchOptions, Suggestion, TextInputOptions, Theme,
        button, icons, provide_theme, search, suggestion,
    },
    text, vstack,
};

const ALL: [&str; 3] = ["Wi-Fi", "Bluetooth", "Displays"];
const BAR_ORIGIN: f32 = 24.0;
const ROW_STEP: f32 = 32.0;

fn matching(query: &str) -> Vec<Suggestion> {
    ALL.iter()
        .filter(|label| label.to_lowercase().contains(&query.to_lowercase()))
        .map(|label| suggestion(*label))
        .collect()
}

fn filtered_rows(cx: &mut Cx, query: Signal<String>) -> Signal<Vec<Suggestion>> {
    let rows = cx.signal(matching(""));
    cx.effect(move |runtime| {
        let query = query.get(runtime);
        rows.set(runtime, matching(&query));
    });
    rows
}

fn app(configure: fn(Element<SearchKind>) -> Element<SearchKind>) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, Theme::dark());
        let query = cx.signal(String::new());
        let picked = cx.signal(String::from("none"));
        let rows = filtered_rows(cx, query);
        vstack((
            configure(
                search(query)
                    .suggestions(rows)
                    .on_select(move |cx, index| picked.set(cx, format!("#{index}"))),
            )
            .on_submit(move |cx, event| picked.set(cx, format!("submit {}", event.value))),
            text(bind(move |runtime| {
                format!("query=[{}]", query.get(runtime))
            })),
            text(bind(move |runtime| {
                format!("picked={}", picked.get(runtime))
            })),
        ))
        .p(BAR_ORIGIN)
    }))
}

fn bar() -> crowntest::Query {
    role(Role::SearchField)
}

fn rows() -> crowntest::Query {
    role(Role::ListItem)
}

fn picked(app: &mut TestApp) -> String {
    app.text_of(text_query("picked="))
}

fn query_of(app: &mut TestApp) -> String {
    app.text_of(text_query("query=["))
}

fn highlighted_row(app: &mut TestApp) -> Option<usize> {
    app.settle();
    let frame = app.frame_snapshot();
    let lines: Vec<&str> = frame.lines().collect();
    let pill = lines
        .iter()
        .position(|line| line.trim_start().starts_with("fill 0,0 242x32 r16"))?;
    let top: f32 = lines[pill - 1]
        .trim()
        .strip_prefix("layer at 5,")?
        .parse()
        .ok()?;
    Some(((top - 5.0) / ROW_STEP) as usize)
}

fn focused_app() -> TestApp {
    let mut app = app(|bar| bar);
    app.focus(bar());
    app.settle();
    app
}

#[test]
fn the_bar_is_a_pill_with_a_magnifier_a_clear_button_and_the_default_placeholder() {
    let mut app = app(|bar| bar);
    let bounds = app.find(bar()).bounds;
    assert_eq!((bounds.size.width, bounds.size.height), (252.0, 34.0));
    let frame = app.frame_snapshot();
    assert!(frame.contains("fill 0,0 252x34 r17"), "{frame}");
    assert!(frame.contains("text \"Search…\""), "{frame}");
    assert!(frame.contains("stroke1.9"), "{frame}");
    assert!(app.exists(role(Role::ComboBox)));
    assert!(app.exists(role(Role::SearchField).name("Search…")));
}

#[test]
fn focusing_the_bar_unfolds_its_suggestions() {
    let mut app = app(|bar| bar);
    assert_eq!(
        app.count(rows()),
        0,
        "nothing shows until the bar is focused"
    );
    app.focus(bar());
    app.advance_ms(16);
    assert!(app.is_animating());
    app.settle();
    assert_eq!(app.count(rows()), 3);
    assert!(
        app.frame_snapshot()
            .contains("fill 0,0 252x106 r21 #1d1d1dff")
    );
}

#[test]
fn nothing_to_suggest_means_no_popup() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let query = cx.signal(String::new());
        vstack(search(query)).p(BAR_ORIGIN)
    }));
    app.focus(bar());
    app.settle();
    assert_eq!(app.count(rows()), 0);
}

#[test]
fn losing_focus_folds_the_popup_away() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let query = cx.signal(String::new());
        let rows = filtered_rows(cx, query);
        vstack((search(query).suggestions(rows), button("Elsewhere"))).p(BAR_ORIGIN)
    }));
    app.focus(bar());
    app.settle();
    assert!(app.frame_snapshot().contains("text \"Bluetooth\""));
    app.focus(role(Role::Button).name("Elsewhere"));
    app.advance_ms(16);
    assert!(app.is_animating(), "folding away takes a moment");
    assert!(app.frame_snapshot().contains("text \"Bluetooth\""));
    app.settle();
    assert!(!app.frame_snapshot().contains("text \"Bluetooth\""));
    assert_eq!(app.count(rows()), 0);
}

#[test]
fn the_panel_hangs_below_the_bar_as_wide_as_it() {
    let mut app = focused_app();
    let first = app.find(rows().name("Wi-Fi")).bounds;
    assert_eq!(first.origin.x, BAR_ORIGIN + 5.0);
    assert_eq!(first.origin.y, BAR_ORIGIN + 34.0 + 6.0 + 5.0);
    assert_eq!(first.size.width, 252.0 - 10.0);
    assert_eq!(first.size.height, 32.0);
    let last = app.find(rows().name("Displays")).bounds;
    assert_eq!(last.origin.y, first.origin.y + 2.0 * 32.0);
}

#[test]
fn arrow_keys_walk_the_rows_and_wrap() {
    let mut app = focused_app();
    assert_eq!(highlighted_row(&mut app), None);
    app.press_key("ArrowDown");
    assert_eq!(highlighted_row(&mut app), Some(0));
    app.press_key("ArrowDown");
    app.press_key("ArrowDown");
    assert_eq!(highlighted_row(&mut app), Some(2));
    app.press_key("ArrowDown");
    assert_eq!(
        highlighted_row(&mut app),
        Some(0),
        "past the last row is the first"
    );
    app.press_key("ArrowUp");
    assert_eq!(highlighted_row(&mut app), Some(2));
}

#[test]
fn up_from_nothing_lands_on_the_last_row() {
    let mut app = focused_app();
    app.press_key("ArrowUp");
    assert_eq!(highlighted_row(&mut app), Some(2));
}

#[test]
fn the_highlight_glides_between_rows_with_a_spring() {
    let mut app = focused_app();
    app.press_key("ArrowDown");
    app.settle();
    app.press_key("ArrowDown");
    app.advance_ms(16);
    assert!(app.is_animating());
    app.settle();
    assert_eq!(highlighted_row(&mut app), Some(1));
}

#[test]
fn enter_picks_the_highlighted_row_and_puts_the_popup_away() {
    let mut app = focused_app();
    app.press_key("ArrowDown");
    app.press_key("ArrowDown");
    app.press_key("Enter");
    assert_eq!(picked(&mut app), "picked=#1");
    app.settle();
    assert_eq!(app.count(rows()), 0);
}

#[test]
fn enter_with_nothing_highlighted_submits_the_query() {
    let mut app = focused_app();
    app.type_text("bl");
    app.press_key("Enter");
    assert_eq!(picked(&mut app), "picked=submit bl");
    app.settle();
    assert_eq!(app.count(rows()), 0);
}

#[test]
fn escape_dismisses_and_an_arrow_brings_it_back() {
    let mut app = focused_app();
    app.press_key("Escape");
    app.settle();
    assert_eq!(app.count(rows()), 0);
    assert!(
        app.find(bar()).states.focused,
        "dismissing the popup is not leaving the bar"
    );
    app.press_key("ArrowDown");
    app.settle();
    assert_eq!(app.count(rows()), 3);
    assert_eq!(highlighted_row(&mut app), Some(0));
}

#[test]
fn typing_after_an_escape_brings_the_popup_back() {
    let mut app = focused_app();
    app.press_key("Escape");
    app.settle();
    app.type_text("i");
    app.settle();
    assert_eq!(app.count(rows()), 2);
}

#[test]
fn escape_only_belongs_to_the_popup_while_it_is_open() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let query = cx.signal(String::new());
        let escapes = cx.signal(0);
        vstack((
            search(query),
            text(bind(move |runtime| {
                format!("escapes={}", escapes.get(runtime))
            })),
        ))
        .on_key_down(move |cx, event| {
            if event.key == input::Key::Named(input::NamedKey::Escape) {
                escapes.update(cx, |count| *count += 1);
            }
        })
    }));
    app.focus(bar());
    app.press_key("Escape");
    assert_eq!(app.text_of(text_query("escapes=")), "escapes=1");
}

#[test]
fn max_suggestions_caps_the_panel_and_the_walk() {
    let mut app = app(|bar| bar.max_suggestions(2));
    app.focus(bar());
    app.settle();
    assert_eq!(app.count(rows()), 2);
    app.press_key("ArrowUp");
    assert_eq!(highlighted_row(&mut app), Some(1));
}

#[test]
fn new_rows_replace_an_open_popup_in_place() {
    let mut app = focused_app();
    app.type_text("bl");
    assert_eq!(app.count(rows()), 1);
    assert!(app.exists(rows().name("Bluetooth")));
    app.advance_ms(16);
    assert!(
        !app.frame_snapshot().contains("opacity"),
        "an update does not unfold again: {}",
        app.frame_snapshot()
    );
}

#[test]
fn new_rows_clear_the_highlight_because_an_old_index_means_nothing_now() {
    let mut app = focused_app();
    app.press_key("ArrowDown");
    assert_eq!(highlighted_row(&mut app), Some(0));
    app.type_text("bl");
    assert_eq!(highlighted_row(&mut app), None);
}

#[test]
fn running_out_of_rows_folds_the_old_ones_away() {
    let mut app = focused_app();
    app.type_text("zzz");
    app.advance_ms(16);
    assert!(app.is_animating());
    assert!(
        app.frame_snapshot().contains("text \"Displays\""),
        "the fold-away shows what was there, not an empty panel"
    );
    app.settle();
    assert!(!app.frame_snapshot().contains("text \"Displays\""));
}

#[test]
fn clicking_a_row_picks_it_and_keeps_the_caret_in_the_bar() {
    let mut app = focused_app();
    let center = app.find(rows().name("Bluetooth")).bounds.center();
    app.pointer_move(center.x, center.y);
    app.pointer_down(center.x, center.y);
    app.pointer_up(center.x, center.y);
    assert_eq!(picked(&mut app), "picked=#1");
    assert!(app.find(bar()).states.focused);
}

#[test]
fn hovering_a_row_highlights_it_and_enter_picks_what_is_visible() {
    let mut app = focused_app();
    let center = app.find(rows().name("Displays")).bounds.center();
    app.pointer_move(center.x, center.y);
    assert_eq!(highlighted_row(&mut app), Some(2));
    app.press_key("Enter");
    assert_eq!(picked(&mut app), "picked=#2");
}

#[test]
fn the_bar_stays_interactive_while_the_popup_is_up() {
    let mut app = focused_app();
    app.pointer_down(BAR_ORIGIN + 100.0, BAR_ORIGIN + 17.0);
    app.pointer_up(BAR_ORIGIN + 100.0, BAR_ORIGIN + 17.0);
    assert!(app.find(bar()).states.focused);
    app.settle();
    assert_eq!(app.count(rows()), 3);
}

#[test]
fn a_click_in_the_panel_padding_does_not_steal_focus() {
    let mut app = focused_app();
    app.pointer_down(BAR_ORIGIN + 2.0, BAR_ORIGIN + 34.0 + 6.0 + 2.0);
    app.pointer_up(BAR_ORIGIN + 2.0, BAR_ORIGIN + 34.0 + 6.0 + 2.0);
    assert!(app.find(bar()).states.focused);
}

#[test]
fn without_on_select_a_pick_fills_the_query_with_the_caret_after_it() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let query = cx.signal(String::from("b"));
        let rows = filtered_rows(cx, query);
        vstack((
            search(query).suggestions(rows),
            text(bind(move |runtime| {
                format!("query=[{}]", query.get(runtime))
            })),
        ))
    }));
    app.focus(bar());
    app.settle();
    app.click(rows().name("Bluetooth"));
    assert_eq!(query_of(&mut app), "query=[Bluetooth]");
    app.settle();
    assert_eq!(app.count(rows()), 0, "a pick puts the popup away");
    app.type_text("!");
    assert_eq!(query_of(&mut app), "query=[Bluetooth!]");
}

#[test]
fn the_clear_button_empties_the_query() {
    let mut app = focused_app();
    app.type_text("bl");
    let x = BAR_ORIGIN + 252.0 - 10.0 - 7.5;
    app.pointer_down(x, BAR_ORIGIN + 17.0);
    app.pointer_up(x, BAR_ORIGIN + 17.0);
    assert_eq!(query_of(&mut app), "query=[]");
}

#[test]
fn field_options_reach_the_bar_underneath() {
    let mut app = app(|bar| {
        bar.placeholder("Find a setting")
            .shape(FieldShape::Rounded)
            .clear_button(ClearButton::Hidden)
            .width(300.0)
    });
    let bounds = app
        .find(role(Role::SearchField).name("Find a setting"))
        .bounds;
    assert_eq!(bounds.size.width, 300.0);
    let frame = app.frame_snapshot();
    assert!(frame.contains("fill 0,0 300x34 r10"), "{frame}");
}

#[test]
fn a_stretched_bar_fills_its_column_and_the_panel_follows() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let query = cx.signal(String::new());
        let rows = filtered_rows(cx, query);
        vstack(search(query).suggestions(rows).stretch())
            .p(BAR_ORIGIN)
            .w(400.0)
    }));
    assert_eq!(app.find(bar()).bounds.size.width, 352.0);
    app.focus(bar());
    app.settle();
    assert_eq!(app.find(rows().name("Wi-Fi")).bounds.size.width, 342.0);
}

#[test]
fn a_disabled_bar_cannot_be_focused_and_never_opens() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let query = cx.signal(String::new());
        let rows = filtered_rows(cx, query);
        vstack(search(query).suggestions(rows).disabled(true)).p(BAR_ORIGIN)
    }));
    assert!(app.find(bar()).states.disabled);
    app.pointer_down(BAR_ORIGIN + 100.0, BAR_ORIGIN + 17.0);
    app.pointer_up(BAR_ORIGIN + 100.0, BAR_ORIGIN + 17.0);
    app.settle();
    assert_eq!(app.count(rows()), 0);
}

fn detailed_app(width: f32) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, Theme::dark());
        let query = cx.signal(String::new());
        let rows = cx.signal(vec![
            suggestion("Wi-Fi").detail("Network").icon(icons::WIFI),
        ]);
        vstack(search(query).suggestions(rows).width(width)).p(BAR_ORIGIN)
    }))
}

#[test]
fn a_detail_is_shown_whole_beside_the_label_or_not_at_all() {
    let mut wide = detailed_app(252.0);
    wide.focus(bar());
    wide.settle();
    assert!(wide.frame_snapshot().contains("text \"Network\""));

    let mut narrow = detailed_app(130.0);
    narrow.focus(bar());
    narrow.settle();
    let frame = narrow.frame_snapshot();
    assert!(frame.contains("text \"Wi-Fi\""), "{frame}");
    assert!(!frame.contains("text \"Network\""), "{frame}");
}

#[test]
fn switching_themes_repaints_the_panel() {
    let mut app = TestApp::new(component(|cx| {
        let theme = provide_theme(cx, Theme::light());
        let query = cx.signal(String::new());
        let rows = filtered_rows(cx, query);
        vstack((
            search(query).suggestions(rows),
            button("Switch").on_click(move |cx, _| theme.set(cx, Theme::dark())),
        ))
        .p(BAR_ORIGIN)
    }));
    app.focus(bar());
    app.settle();
    assert!(!app.frame_snapshot().contains("#1d1d1dff"));
    app.press_key("Tab");
    app.click(role(Role::Button).name("Switch"));
    app.focus(bar());
    app.settle();
    assert!(
        app.frame_snapshot().contains("#1d1d1dff"),
        "{}",
        app.frame_snapshot()
    );
}

#[test]
fn an_open_popup_with_icons_details_and_a_highlight_keeps_its_look() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let query = cx.signal(String::new());
        let rows = cx.signal(vec![
            suggestion("Wi-Fi").icon(icons::WIFI).detail("Network"),
            suggestion("Bluetooth").icon(icons::BLUETOOTH),
            suggestion("A label far too long to sit beside its detail text").detail("Display"),
        ]);
        vstack(search(query).suggestions(rows)).p(BAR_ORIGIN)
    }));
    app.focus(bar());
    app.press_key("ArrowDown");
    app.press_key("ArrowDown");
    app.settle();
    insta::assert_snapshot!(app.snapshot());
}
