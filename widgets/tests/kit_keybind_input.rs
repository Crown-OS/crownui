use crownconfig::{KeyCode, Keybind, Mods};
use crowntest::{Role, TestApp, role, text as text_query};
use input::{Key, KeyboardEvent, Modifiers, NamedKey, PhysicalKey};
use view::{Listeners, Semantic, Styled, bind, component};
use widgets::{
    kit::{KeybindInputOptions, Theme, button, keybind_input, provide_theme},
    text, vstack,
};
use windowing::WindowEvent;

type Strike = (Key, u32);

const LEFT_CTRL: Strike = (Key::Named(NamedKey::Control), 29);
const LEFT_ALT: Strike = (Key::Named(NamedKey::Alt), 56);
const LEFT_SHIFT: Strike = (Key::Named(NamedKey::Shift), 42);
const RIGHT_SHIFT: Strike = (Key::Named(NamedKey::Shift), 54);
const LEFT_META: Strike = (Key::Named(NamedKey::Meta), 125);
const KEY_D: Strike = (Key::Character('d'), 32);
const ESCAPE: Strike = (Key::Named(NamedKey::Escape), 1);
const BACKSPACE: Strike = (Key::Named(NamedKey::Backspace), 14);
const KEYPAD_ENTER: Strike = (Key::Named(NamedKey::Enter), 96);
const FIELD_ORIGIN: f32 = 24.0;

fn event((key, code): Strike, repeat: bool) -> KeyboardEvent {
    KeyboardEvent {
        key,
        code: PhysicalKey(code),
        text: None,
        repeat,
        modifiers: Modifiers::NONE,
    }
}

fn send(app: &mut TestApp, window_event: WindowEvent) {
    if let Err(error) = app.inline().handle(&window_event) {
        panic!("frame failed: {error}");
    }
}

fn down(app: &mut TestApp, strike: Strike) {
    send(app, WindowEvent::KeyDown(event(strike, false)));
}

fn up(app: &mut TestApp, strike: Strike) {
    send(app, WindowEvent::KeyUp(event(strike, false)));
}

fn chord(app: &mut TestApp, strikes: &[Strike]) {
    for strike in strikes {
        down(app, *strike);
    }
    for strike in strikes.iter().rev() {
        up(app, *strike);
    }
}

fn mods(ctrl: bool, alt: bool, shift: bool, meta: bool) -> Mods {
    Mods {
        meta,
        ctrl,
        alt,
        shift,
    }
}

fn app(initial: Keybind) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, Theme::dark());
        let bind_signal = cx.signal(initial);
        let changes = cx.signal(String::from("none"));
        vstack((
            keybind_input(bind_signal)
                .on_change(move |cx, event| changes.set(cx, event.value.clone())),
            text(bind(move |runtime| {
                format!("bound={}", bind_signal.get(runtime))
            })),
            text(bind(move |runtime| {
                format!("changed={}", changes.get(runtime))
            })),
            button("Elsewhere"),
        ))
        .p(FIELD_ORIGIN)
        .gap(8.0)
    }))
}

fn field() -> crowntest::Query {
    role(Role::TextField)
}

fn bound(app: &mut TestApp) -> String {
    app.text_of(text_query("bound="))
}

fn changed(app: &mut TestApp) -> String {
    app.text_of(text_query("changed="))
}

fn armed(initial: Keybind) -> TestApp {
    let mut app = app(initial);
    app.click(field());
    app
}

fn shows(app: &mut TestApp, label: &str) -> bool {
    app.frame_snapshot().contains(&format!("text \"{label}\""))
}

#[test]
fn the_field_is_a_34px_plate_wide_as_a_select_trigger() {
    let mut app = app(Keybind::SUPER_SPACE);
    let bounds = app.find(field()).bounds;
    assert_eq!((bounds.size.width, bounds.size.height), (252.0, 34.0));
    assert!(
        app.frame_snapshot()
            .contains("fill 0,0 252x34 r10 #1c1c1cff")
    );
}

#[test]
fn the_chord_is_drawn_as_one_cap_per_part_and_none_means_a_hint() {
    let mut bound_app = app(Keybind::SUPER_SPACE);
    assert!(shows(&mut bound_app, "Super") && shows(&mut bound_app, "Space"));
    assert!(!shows(&mut bound_app, "None"));
    let mut unbound = app(Keybind::NONE);
    assert!(shows(&mut unbound, "None"));
    assert!(!shows(&mut unbound, "Super"));
}

#[test]
fn caps_are_22px_plates_cut_with_a_6px_corner_and_centered_as_a_group() {
    let mut app = app(Keybind::SUPER_SPACE);
    let frame = app.frame_snapshot();
    assert!(
        frame.contains("fill 0,0 52.2x22 r6") || frame.contains("fill 0,0 52x22 r6"),
        "{frame}"
    );
    let layout = app.layout_snapshot();
    assert!(layout.contains("@95.5,30"), "{layout}");
}

#[test]
fn a_field_at_rest_ignores_the_keyboard() {
    let mut app = app(Keybind::SUPER_SPACE);
    app.focus(field());
    chord(&mut app, &[LEFT_CTRL, KEY_D]);
    assert_eq!(bound(&mut app), "bound=Super+Space");
}

#[test]
fn clicking_arms_and_a_second_click_disarms() {
    let mut app = app(Keybind::SUPER_SPACE);
    app.click(field());
    assert!(shows(&mut app, "Press a shortcut…"));
    assert!(!shows(&mut app, "Super"));
    app.click(field());
    assert!(!shows(&mut app, "Press a shortcut…"));
    assert!(shows(&mut app, "Super"));
}

#[test]
fn a_real_pointer_click_focuses_and_arms_the_field() {
    let mut app = app(Keybind::NONE);
    app.pointer_down(FIELD_ORIGIN + 100.0, FIELD_ORIGIN + 17.0);
    app.pointer_up(FIELD_ORIGIN + 100.0, FIELD_ORIGIN + 17.0);
    assert!(app.find(field()).states.focused);
    assert!(shows(&mut app, "Press a shortcut…"));
}

#[test]
fn enter_on_the_focused_field_arms_it_for_keyboard_users() {
    let mut app = app(Keybind::NONE);
    app.focus(field());
    app.press_key("Enter");
    assert!(shows(&mut app, "Press a shortcut…"));
}

#[test]
fn striking_a_key_commits_the_chord_it_was_struck_with() {
    let mut app = armed(Keybind::NONE);
    chord(&mut app, &[LEFT_CTRL, LEFT_ALT, KEY_D]);
    assert_eq!(bound(&mut app), "bound=Ctrl+Alt+D");
    assert_eq!(changed(&mut app), "changed=Ctrl+Alt+D");
    assert!(!shows(&mut app, "Press a shortcut…"));
    assert!(shows(&mut app, "Ctrl") && shows(&mut app, "Alt") && shows(&mut app, "D"));
}

#[test]
fn the_caps_build_up_while_modifiers_are_held() {
    let mut app = armed(Keybind::NONE);
    down(&mut app, LEFT_META);
    assert!(shows(&mut app, "Super") && shows(&mut app, "…"));
    assert!(!shows(&mut app, "Press a shortcut…"));
    down(&mut app, LEFT_SHIFT);
    assert!(shows(&mut app, "Shift"));
}

#[test]
fn releasing_the_last_modifier_commits_a_modifier_only_chord() {
    let mut app = armed(Keybind::NONE);
    down(&mut app, LEFT_META);
    assert_eq!(bound(&mut app), "bound=None", "not until it comes up");
    up(&mut app, LEFT_META);
    assert_eq!(bound(&mut app), "bound=Super");
}

#[test]
fn a_modifier_is_held_until_every_key_that_sets_it_is_up() {
    let mut app = armed(Keybind::NONE);
    down(&mut app, LEFT_SHIFT);
    down(&mut app, RIGHT_SHIFT);
    up(&mut app, LEFT_SHIFT);
    assert_eq!(bound(&mut app), "bound=None", "shift is still down");
    up(&mut app, RIGHT_SHIFT);
    assert_eq!(bound(&mut app), "bound=Shift");
}

#[test]
fn a_modifier_held_before_the_field_was_armed_still_counts() {
    let mut app = armed(Keybind::NONE);
    let mut struck = event(KEY_D, false);
    struck.modifiers.alt_key = true;
    send(&mut app, WindowEvent::KeyDown(struck));
    assert_eq!(bound(&mut app), "bound=Alt+D");
}

#[test]
fn escape_keeps_backspace_clears_and_a_modifier_turns_both_into_keys() {
    let mut app = armed(Keybind::SUPER_SPACE);
    chord(&mut app, &[ESCAPE]);
    assert_eq!(bound(&mut app), "bound=Super+Space");
    assert!(!shows(&mut app, "Press a shortcut…"));

    app.click(field());
    chord(&mut app, &[BACKSPACE]);
    assert_eq!(bound(&mut app), "bound=None");
    assert_eq!(changed(&mut app), "changed=None");

    app.click(field());
    chord(&mut app, &[LEFT_CTRL, ESCAPE]);
    assert_eq!(bound(&mut app), "bound=Ctrl+Escape");
}

#[test]
fn losing_focus_disarms_the_field_and_keeps_the_value() {
    let mut app = armed(Keybind::SUPER_SPACE);
    app.focus(role(Role::Button).name("Elsewhere"));
    assert!(!shows(&mut app, "Press a shortcut…"));
    assert_eq!(bound(&mut app), "bound=Super+Space");
}

#[test]
fn a_key_no_shortcut_may_use_is_not_recorded_and_the_field_stays_armed() {
    let mut app = armed(Keybind::NONE);
    chord(&mut app, &[KEYPAD_ENTER]);
    assert_eq!(bound(&mut app), "bound=None");
    assert!(shows(&mut app, "Press a shortcut…"));
}

#[test]
fn an_armed_field_swallows_every_key_so_tab_cannot_leave() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let value = cx.signal(Keybind::NONE);
        let seen = cx.signal(0);
        vstack((
            keybind_input(value),
            text(bind(move |runtime| format!("seen={}", seen.get(runtime)))),
        ))
        .on_key_down(move |cx, _| seen.update(cx, |count| *count += 1))
    }));
    app.click(field());
    chord(&mut app, &[(Key::Named(NamedKey::Tab), 15)]);
    assert!(app.find(field()).states.focused);
    assert_eq!(app.text_of(text_query("seen=")), "seen=0");
}

#[test]
fn key_repeat_says_nothing_new() {
    let mut app = armed(Keybind::NONE);
    send(&mut app, WindowEvent::KeyDown(event(KEY_D, true)));
    assert_eq!(bound(&mut app), "bound=None");
    assert!(shows(&mut app, "Press a shortcut…"));
}

#[test]
fn a_value_set_from_outside_cancels_recording() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let value = cx.signal(Keybind::NONE);
        vstack((
            keybind_input(value),
            button("Preset").on_click(move |cx, _| value.set(cx, Keybind::SUPER_SPACE)),
        ))
    }));
    app.click(field());
    assert!(shows(&mut app, "Press a shortcut…"));
    app.click(role(Role::Button).name("Preset"));
    assert!(!shows(&mut app, "Press a shortcut…"));
    assert!(shows(&mut app, "Super"));
}

#[test]
fn focus_turns_the_border_accent_with_a_spring_and_no_halo() {
    let mut app = app(Keybind::SUPER_SPACE);
    app.focus(field());
    app.advance_ms(16);
    assert!(app.is_animating());
    app.settle();
    let focused = app.frame_snapshot();
    assert!(
        focused.contains("stroke1 0.5,0.5 251x33 r9.5 gradient("),
        "{focused}"
    );
    assert!(!focused.contains("stroke3"), "{focused}");
}

#[test]
fn a_disabled_field_cannot_be_armed_and_dims_its_caps() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let value = cx.signal(Keybind::SUPER_SPACE);
        vstack(keybind_input(value).disabled(true)).p(FIELD_ORIGIN)
    }));
    assert!(app.find(field()).states.disabled);
    app.pointer_down(FIELD_ORIGIN + 100.0, FIELD_ORIGIN + 17.0);
    app.pointer_up(FIELD_ORIGIN + 100.0, FIELD_ORIGIN + 17.0);
    assert!(!shows(&mut app, "Press a shortcut…"));
    assert!(shows(&mut app, "Super"));
}

#[test]
fn the_field_announces_the_chord_and_says_what_it_wants_while_armed() {
    let mut app = app(Keybind::SUPER_SPACE);
    assert_eq!(app.find(field()).value.as_deref(), Some("Super+Space"));
    app.click(field());
    assert_eq!(
        app.find(field()).value.as_deref(),
        Some("Press a shortcut…")
    );
}

#[test]
fn width_sets_the_field_and_a_long_chord_stays_inside_it() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let value = cx.signal(Keybind::new(
            mods(true, true, true, true),
            Some(KeyCode::Backquote),
        ));
        vstack(keybind_input(value).width(120.0)).p(FIELD_ORIGIN)
    }));
    assert_eq!(app.find(field()).bounds.size.width, 120.0);
    app.advance_ms(16);
    let layout = app.layout_snapshot();
    assert!(
        layout.contains("for_each @34,30 "),
        "left-aligned at the padding, not centered off the edge: {layout}"
    );
}

#[test]
fn switching_themes_repaints_the_plate() {
    let mut app = TestApp::new(component(|cx| {
        let theme = provide_theme(cx, Theme::light());
        let value = cx.signal(Keybind::SUPER_SPACE);
        vstack((
            keybind_input(value),
            button("Switch").on_click(move |cx, _| theme.set(cx, Theme::dark())),
        ))
        .p(FIELD_ORIGIN)
    }));
    assert!(!app.frame_snapshot().contains("#1c1c1cff"));
    app.click(role(Role::Button).name("Switch"));
    app.settle();
    assert!(app.frame_snapshot().contains("#1c1c1cff"));
}

#[test]
fn a_composed_set_of_chords_keeps_its_look() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let (bound_chord, unbound, modifiers_only) = (
            cx.signal(Keybind::SUPER_SPACE),
            cx.signal(Keybind::NONE),
            cx.signal(Keybind::SUPER_CTRL),
        );
        vstack((
            keybind_input(bound_chord),
            keybind_input(unbound),
            keybind_input(modifiers_only),
        ))
        .p(FIELD_ORIGIN)
        .gap(12.0)
    }));
    insta::assert_snapshot!(app.snapshot());
}
