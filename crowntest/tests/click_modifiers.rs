use crowntest::{Role, TestApp, role};
use input::Modifiers;
use view::{Listeners, View, bind, component};
use widgets::{button, text, vstack};

fn selector() -> impl View {
    component(|cx| {
        let last = cx.signal(String::from("none"));
        vstack((
            text(bind(move |rt| last.get(rt))),
            button("file").on_click(move |cx, event| {
                let held = event.modifiers;
                let mode = match (held.ctrl_key, held.shift_key) {
                    (true, _) => "toggle",
                    (_, true) => "range",
                    _ => "single",
                };
                last.set(cx, mode.to_owned());
            }),
        ))
    })
}

#[test]
fn a_click_carries_the_modifiers_held_on_the_keyboard() {
    let mut app = TestApp::new(selector());
    let file = || role(Role::Button).name("file");
    app.click(file());
    assert_eq!(app.text_of(role(Role::Text)), "single");
    app.hold_modifiers(Modifiers {
        ctrl_key: true,
        ..Modifiers::NONE
    })
    .click(file());
    assert_eq!(app.text_of(role(Role::Text)), "toggle");
    app.hold_modifiers(Modifiers {
        shift_key: true,
        ..Modifiers::NONE
    })
    .click(file());
    assert_eq!(app.text_of(role(Role::Text)), "range");
    app.hold_modifiers(Modifiers::NONE).click(file());
    assert_eq!(app.text_of(role(Role::Text)), "single");
}

#[test]
fn a_pointer_click_reads_the_modifiers_on_its_events() {
    let mut app = TestApp::new(selector());
    let center = app.find(role(Role::Button).name("file")).bounds.center();
    app.hold_modifiers(Modifiers {
        shift_key: true,
        ..Modifiers::NONE
    });
    app.pointer_down(center.x, center.y)
        .pointer_up(center.x, center.y);
    assert_eq!(app.text_of(role(Role::Text)), "single");
}
