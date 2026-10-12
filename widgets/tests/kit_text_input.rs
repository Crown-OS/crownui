use crowntest::{Role, TestApp, role, text as text_query};
use input::TextInputEvent;
use view::{ClipboardRequest, Element, Listeners, Semantic, Styled, bind, component};
use widgets::{
    kit::{
        ClearButton, FieldShape, TextInputKind, TextInputOptions, Theme, icons, provide_theme,
        text_input,
    },
    text, vstack,
};
use windowing::WindowEvent;

const FIELD_ORIGIN: f32 = 24.0;

fn app(
    initial: &'static str,
    configure: fn(Element<TextInputKind>) -> Element<TextInputKind>,
) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, Theme::dark());
        let value = cx.signal(initial.to_owned());
        vstack((
            configure(text_input(value)),
            text(bind(move |runtime| {
                format!("value=[{}]", value.get(runtime))
            })),
        ))
        .p(FIELD_ORIGIN)
        .gap(8.0)
    }))
}

fn field() -> crowntest::Query {
    role(Role::TextField)
}

fn echoed(app: &mut TestApp) -> String {
    app.text_of(text_query("value=["))
}

fn typed(app: &mut TestApp, text: &str) {
    app.focus(field());
    app.type_text(text);
}

#[test]
fn the_field_is_a_34px_pill_wide_as_a_select_trigger() {
    let mut app = app("", |field| field);
    let bounds = app.find(field()).bounds;
    assert_eq!((bounds.size.width, bounds.size.height), (252.0, 34.0));
    assert!(
        app.frame_snapshot()
            .contains("fill 0,0 252x34 r10 #1c1c1cff")
    );
}

#[test]
fn a_pill_field_rounds_its_ends_fully() {
    let app = app("", |field| field.shape(FieldShape::Pill));
    assert!(app.frame_snapshot().contains("fill 0,0 252x34 r17"));
}

#[test]
fn width_and_stretch_size_the_field() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let value = cx.signal(String::new());
        vstack((
            text_input(value).label("Narrow").width(120.0),
            text_input(value).label("Wide").stretch(),
        ))
        .p(24.0)
        .w(400.0)
    }));
    assert_eq!(
        app.find(role(Role::TextField).name("Narrow"))
            .bounds
            .size
            .width,
        120.0
    );
    assert_eq!(
        app.find(role(Role::TextField).name("Wide"))
            .bounds
            .size
            .width,
        352.0
    );
}

#[test]
fn typing_inserts_at_the_caret_and_fires_input_with_the_whole_value() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let value = cx.signal(String::from("ac"));
        let heard = cx.signal(String::new());
        vstack((
            text_input(value).on_input(move |cx, event| heard.set(cx, event.value.clone())),
            text(bind(move |runtime| {
                format!("heard=[{}]", heard.get(runtime))
            })),
        ))
    }));
    app.focus(field());
    app.press_key("ArrowLeft");
    app.type_text("b");
    assert_eq!(app.text_of(text_query("heard=[")), "heard=[abc]");
}

#[test]
fn keys_that_print_insert_and_space_inserts_a_space() {
    let mut app = app("", |field| field);
    app.focus(field());
    app.press_key("h");
    app.press_key("i");
    app.press_key("Space");
    app.press_key("Home");
    app.press_key("Space");
    assert_eq!(echoed(&mut app), "value=[ hi ]");
}

#[test]
fn backspace_and_delete_remove_one_grapheme_each_side_of_the_caret() {
    let mut app = app("abcd", |field| field);
    app.focus(field());
    app.press_key("ArrowLeft");
    app.press_key("Backspace");
    assert_eq!(echoed(&mut app), "value=[abd]");
    app.press_key("Delete");
    assert_eq!(echoed(&mut app), "value=[ab]");
    app.press_key("Delete");
    assert_eq!(echoed(&mut app), "value=[ab]");
}

#[test]
fn home_and_end_move_the_caret_to_the_edges() {
    let mut app = app("bc", |field| field);
    app.focus(field());
    app.press_key("Home");
    app.type_text("a");
    app.press_key("End");
    app.type_text("d");
    assert_eq!(echoed(&mut app), "value=[abcd]");
}

#[test]
fn shift_arrows_select_and_typing_replaces_the_selection() {
    let mut app = app("hello world", |field| field);
    app.focus(field());
    for _ in 0..5 {
        app.press_key("Shift+ArrowLeft");
    }
    app.type_text("there");
    assert_eq!(echoed(&mut app), "value=[hello there]");
}

#[test]
fn an_arrow_collapses_a_selection_to_its_edge() {
    let mut app = app("abc", |field| field);
    app.focus(field());
    app.press_key("Ctrl+A");
    app.press_key("ArrowLeft");
    app.type_text("x");
    assert_eq!(echoed(&mut app), "value=[xabc]");
}

#[test]
fn control_arrows_and_backspace_work_by_word() {
    let mut app = app("one two three", |field| field);
    app.focus(field());
    app.press_key("Ctrl+Backspace");
    assert_eq!(echoed(&mut app), "value=[one two ]");
    app.press_key("Ctrl+ArrowLeft");
    app.type_text("|");
    assert_eq!(echoed(&mut app), "value=[one |two ]");
    app.press_key("Ctrl+ArrowRight");
    app.type_text("|");
    assert_eq!(echoed(&mut app), "value=[one |two| ]");
}

#[test]
fn select_all_then_backspace_empties_the_field() {
    let mut app = app("abc", |field| field);
    app.focus(field());
    app.press_key("Ctrl+A");
    app.press_key("Backspace");
    assert_eq!(echoed(&mut app), "value=[]");
}

#[test]
fn a_combining_sequence_is_one_character_to_the_caret() {
    let mut app = app("e\u{301}x", |field| field);
    app.focus(field());
    app.press_key("Home");
    app.press_key("Delete");
    assert_eq!(echoed(&mut app), "value=[x]");
}

#[test]
fn pasted_line_breaks_never_reach_a_single_line_field() {
    let mut app = app("", |field| field);
    typed(&mut app, "a\nb");
    assert_eq!(echoed(&mut app), "value=[ab]");
}

#[test]
fn clipboard_shortcuts_copy_cut_and_ask_for_a_paste() {
    let mut app = app("hello", |field| field);
    app.focus(field());
    app.press_key("Ctrl+C");
    assert!(app.ui().take_clipboard_requests().is_empty());
    app.press_key("Ctrl+A");
    app.press_key("Ctrl+C");
    assert_eq!(
        app.ui().take_clipboard_requests(),
        [ClipboardRequest::Copy("hello".into())]
    );
    app.press_key("Ctrl+X");
    assert_eq!(
        app.ui().take_clipboard_requests(),
        [ClipboardRequest::Copy("hello".into())]
    );
    assert_eq!(echoed(&mut app), "value=[]");
    app.press_key("Ctrl+V");
    assert_eq!(
        app.ui().take_clipboard_requests(),
        [ClipboardRequest::Paste]
    );
}

#[test]
fn enter_commits_and_submits_the_value() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let value = cx.signal(String::from("abc"));
        let log = cx.signal(Vec::<String>::new());
        vstack((
            text_input(value)
                .on_change(move |cx, event| {
                    log.update(cx, |log| log.push(format!("change {}", event.value)))
                })
                .on_submit(move |cx, event| {
                    log.update(cx, |log| log.push(format!("submit {}", event.value)))
                }),
            text(bind(move |runtime| {
                log.with(runtime, |log| format!("log=[{}]", log.join(",")))
            })),
        ))
    }));
    app.focus(field());
    app.press_key("Enter");
    assert_eq!(
        app.text_of(text_query("log=[")),
        "log=[change abc,submit abc]"
    );
}

#[test]
fn losing_focus_commits_the_value() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let value = cx.signal(String::from("abc"));
        let committed = cx.signal(0);
        vstack((
            text_input(value).on_change(move |cx, _| committed.update(cx, |count| *count += 1)),
            text(bind(move |runtime| {
                format!("committed={}", committed.get(runtime))
            })),
            widgets::kit::button("Elsewhere"),
        ))
    }));
    app.focus(field());
    app.focus(role(Role::Button).name("Elsewhere"));
    assert_eq!(app.text_of(text_query("committed=")), "committed=1");
}

#[test]
fn a_read_only_field_selects_and_copies_but_never_edits() {
    let mut app = app("fixed", |field| field.read_only());
    app.focus(field());
    app.type_text("x");
    app.press_key("Backspace");
    app.press_key("Ctrl+A");
    app.press_key("Ctrl+C");
    assert_eq!(echoed(&mut app), "value=[fixed]");
    assert_eq!(
        app.ui().take_clipboard_requests(),
        [ClipboardRequest::Copy("fixed".into())]
    );
}

#[test]
fn a_disabled_field_cannot_take_focus_and_dims_its_text() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let value = cx.signal(String::from("locked"));
        vstack(text_input(value).disabled(true)).p(24.0)
    }));
    let node = app.find(field());
    assert!(node.states.disabled && !node.states.focusable);
    assert!(app.frame_snapshot().contains("text \"locked\""));
    app.pointer_down(100.0, 40.0);
    app.pointer_up(100.0, 40.0);
    assert!(!app.find(field()).states.focused);
}

#[test]
fn the_placeholder_shows_only_while_the_field_is_empty() {
    let mut app = app("", |field| field.placeholder("Password"));
    assert!(app.frame_snapshot().contains("text \"Password\""));
    typed(&mut app, "x");
    assert!(!app.frame_snapshot().contains("text \"Password\""));
    app.press_key("Backspace");
    assert!(app.frame_snapshot().contains("text \"Password\""));
}

#[test]
fn the_placeholder_names_the_field_unless_a_label_follows() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let value = cx.signal(String::new());
        vstack((
            text_input(value).placeholder("Network name"),
            text_input(value).placeholder("Ignored").label("Passphrase"),
        ))
    }));
    assert!(app.exists(role(Role::TextField).name("Network name")));
    assert!(app.exists(role(Role::TextField).name("Passphrase")));
}

#[test]
fn focus_turns_the_border_accent_with_a_spring_and_no_halo() {
    let mut app = app("", |field| field);
    app.focus(field());
    app.advance_ms(16);
    assert!(app.is_animating());
    app.settle();
    assert!(!app.is_animating());
    let focused = app.frame_snapshot();
    assert!(!focused.contains("stroke3"), "{focused}");
    assert!(
        focused.contains("stroke1 0.5,0.5 251x33 r9.5 gradient("),
        "{focused}"
    );
}

#[test]
fn blurring_takes_the_accent_border_back_out() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let value = cx.signal(String::new());
        vstack((text_input(value), widgets::kit::button("Elsewhere")))
    }));
    app.focus(field());
    app.settle();
    let accent_border = "stroke1 0.5,0.5 251x33 r9.5 gradient(";
    assert!(app.frame_snapshot().contains(accent_border));
    app.focus(role(Role::Button).name("Elsewhere"));
    app.settle();
    assert!(!app.frame_snapshot().contains(accent_border));
}

#[test]
fn the_caret_is_an_accent_bar_that_exists_only_while_focused() {
    let mut app = app("ab", |field| field);
    assert!(!app.frame_snapshot().contains("fill 0,0 1.5x"));
    app.focus(field());
    app.settle();
    assert!(
        app.frame_snapshot().contains("fill 0,0 1.5x16.8 #6d48e8ff"),
        "{}",
        app.frame_snapshot()
    );
}

#[test]
fn a_selection_is_an_accent_wash_that_dims_when_the_field_loses_focus() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let value = cx.signal(String::from("abc"));
        vstack((text_input(value), widgets::kit::button("Elsewhere")))
    }));
    app.focus(field());
    app.press_key("Ctrl+A");
    assert!(
        app.frame_snapshot().contains("#6d48e859"),
        "{}",
        app.frame_snapshot()
    );
    app.focus(role(Role::Button).name("Elsewhere"));
    app.settle();
    let blurred = app.frame_snapshot();
    assert!(
        blurred.contains("#6d48e82e") && !blurred.contains("#6d48e859"),
        "{blurred}"
    );
}

#[test]
fn a_leading_icon_pushes_the_text_clear_of_it() {
    let mut app = app("hello", |field| field.leading_icon(icons::SEARCH));
    let layout = app.layout_snapshot();
    assert!(layout.contains("@60,32.6"), "{layout}");
}

#[test]
fn the_clear_button_empties_the_field_only_while_it_has_text() {
    let mut app = app("wifi", |field| field.clear_button(ClearButton::Visible));
    let button_x = FIELD_ORIGIN + 252.0 - 10.0 - 7.5;
    let button_y = FIELD_ORIGIN + 17.0;
    app.pointer_down(button_x, button_y);
    app.pointer_up(button_x, button_y);
    assert_eq!(echoed(&mut app), "value=[]");
    assert!(app.find(field()).states.focused);
    app.pointer_down(button_x, button_y);
    app.pointer_up(button_x, button_y);
    assert_eq!(echoed(&mut app), "value=[]");
}

#[test]
fn the_clear_button_is_drawn_only_while_there_is_something_to_clear() {
    let mut app = app("", |field| field.clear_button(ClearButton::Visible));
    let empty = app.frame_snapshot();
    typed(&mut app, "x");
    let filled = app.frame_snapshot();
    assert_ne!(empty, filled);
    assert!(
        filled.matches("svg").count() > empty.matches("svg").count() || filled.len() > empty.len()
    );
}

#[test]
fn a_clear_button_narrows_the_text_column_even_when_hidden() {
    let plain = app("", |field| field).layout_snapshot();
    let clearable = app("", |field| field.clear_button(ClearButton::Visible)).layout_snapshot();
    assert!(plain.contains("stack[2] @34,24 232x34"), "{plain}");
    assert!(clearable.contains("stack[2] @34,24 212x34"), "{clearable}");
}

#[test]
fn a_secret_field_shows_bullets_and_hides_its_value_from_agents() {
    let mut app = app("", |field| field.secret());
    app.focus(field());
    for key in ["h", "u", "n", "t", "e", "r", "2"] {
        app.press_key(key);
    }
    assert_eq!(echoed(&mut app), "value=[hunter2]");
    let frame = app.frame_snapshot();
    assert!(frame.contains("text \"•••••••\""), "{frame}");
    assert!(!frame.contains("text \"hunter2\""), "{frame}");
    let node = app.find(field());
    assert!(node.states.secure && node.exposed_value().is_none());
}

#[test]
fn a_secret_field_masks_clusters_not_scalars_and_refuses_to_copy() {
    let mut app = app("e\u{301}\u{1F468}\u{200D}\u{1F469}x", |field| {
        field.secret()
    });
    assert!(app.frame_snapshot().contains("text \"•••\""));
    app.focus(field());
    app.press_key("Ctrl+A");
    app.press_key("Ctrl+C");
    app.press_key("Ctrl+X");
    assert!(app.ui().take_clipboard_requests().is_empty());
    assert_eq!(
        echoed(&mut app),
        "value=[e\u{301}\u{1F468}\u{200D}\u{1F469}x]"
    );
}

fn click(app: &mut TestApp, x: f32) {
    app.pointer_down(x, 41.0);
    app.pointer_up(x, 41.0);
}

#[test]
fn clicking_places_the_caret_at_the_nearest_boundary() {
    let mut app = app("one two", |field| field);
    click(&mut app, 34.0);
    app.type_text("[");
    click(&mut app, 270.0);
    app.type_text("]");
    assert_eq!(echoed(&mut app), "value=[[one two]]");
}

#[test]
fn a_double_click_selects_a_word_and_a_triple_click_selects_everything() {
    let mut app = app("one two", |field| field);
    click(&mut app, 34.0);
    click(&mut app, 34.0);
    app.type_text("1");
    assert_eq!(echoed(&mut app), "value=[1 two]");
    app.advance_ms(1000);
    click(&mut app, 34.0);
    click(&mut app, 34.0);
    click(&mut app, 34.0);
    app.type_text("x");
    assert_eq!(echoed(&mut app), "value=[x]");
}

#[test]
fn dragging_across_the_text_selects_it() {
    let mut app = app("abcdef", |field| field);
    app.pointer_down(34.0, 41.0);
    app.pointer_move(120.0, 41.0);
    app.pointer_move(270.0, 41.0);
    app.pointer_up(270.0, 41.0);
    app.type_text("z");
    assert_eq!(echoed(&mut app), "value=[z]");
}

#[test]
fn long_text_scrolls_so_the_caret_stays_inside_the_field() {
    let mut app = app("", |field| field);
    typed(&mut app, "the quick brown fox jumps over the lazy dog");
    app.settle();
    let frame = app.frame_snapshot();
    assert!(frame.contains("layer at -"), "{frame}");
    app.press_key("Home");
    app.settle();
    assert!(
        !app.frame_snapshot().contains("layer at -"),
        "{}",
        app.frame_snapshot()
    );
}

#[test]
fn switching_themes_repaints_the_surface() {
    let mut app = TestApp::new(component(|cx| {
        let theme = provide_theme(cx, Theme::light());
        let value = cx.signal(String::new());
        vstack((
            text_input(value),
            widgets::kit::button("Switch").on_click(move |cx, _| theme.set(cx, Theme::dark())),
        ))
        .p(24.0)
    }));
    assert!(!app.frame_snapshot().contains("#1c1c1cff"));
    app.click(role(Role::Button).name("Switch"));
    app.settle();
    assert!(app.frame_snapshot().contains("#1c1c1cff"));
}

#[test]
fn a_composed_form_of_fields_keeps_its_look() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let (name, secret, search) = (
            cx.signal(String::from("Studio")),
            cx.signal(String::from("hunter2")),
            cx.signal(String::new()),
        );
        vstack((
            text_input(name).placeholder("Network name"),
            text_input(secret).placeholder("Password").secret(),
            text_input(search)
                .placeholder("Search…")
                .leading_icon(icons::SEARCH)
                .clear_button(ClearButton::Visible)
                .shape(FieldShape::Pill),
            text_input(name).read_only(),
        ))
        .p(24.0)
        .gap(12.0)
    }));
    insta::assert_snapshot!(app.snapshot());
}

fn deliver(app: &mut TestApp, event: WindowEvent) {
    if let Err(error) = app.inline().handle(&event) {
        panic!("frame failed: {error}");
    }
}

#[test]
fn input_method_events_commit_delete_around_the_caret_and_ignore_composition() {
    let mut app = app("abcdef", |field| field);
    app.focus(field());
    app.press_key("Home");
    app.press_key("ArrowRight");
    app.press_key("ArrowRight");
    app.press_key("ArrowRight");
    deliver(
        &mut app,
        WindowEvent::TextInput(TextInputEvent::DeleteSurrounding {
            before: 1,
            after: 2,
        }),
    );
    assert_eq!(echoed(&mut app), "value=[abf]");
    deliver(
        &mut app,
        WindowEvent::TextInput(TextInputEvent::Preedit {
            text: "ni".into(),
            cursor: None,
        }),
    );
    deliver(
        &mut app,
        WindowEvent::TextInput(TextInputEvent::Commit("你".into())),
    );
    assert_eq!(echoed(&mut app), "value=[ab你f]");
}

#[test]
fn a_pasted_clipboard_inserts_at_the_caret() {
    let mut app = app("ad", |field| field);
    app.focus(field());
    app.press_key("Home");
    app.press_key("ArrowRight");
    app.press_key("Ctrl+V");
    deliver(&mut app, WindowEvent::ClipboardText("bc".into()));
    assert_eq!(echoed(&mut app), "value=[abcd]");
}
