use crowntest::{Role, TestApp, role};
use view::{Cx, NodeId, View, bind, component, show};
use widgets::{text as label, text_field, vstack};

struct Autofocus<V>(V);

impl<V: View> View for Autofocus<V> {
    fn build(self, cx: &mut Cx) -> NodeId {
        let node = self.0.build(cx);
        cx.request_focus(node);
        node
    }
}

fn password_prompt(cx: &mut Cx) -> impl View + use<> {
    let password = cx.signal(String::new());
    vstack((
        Autofocus(text_field(password, "Password")),
        label(bind(move |rt| format!("typed: {}", password.get(rt)))),
    ))
}

#[test]
fn a_field_focused_on_mount_receives_typing_without_a_pointer_event() {
    let mut app = TestApp::new(component(password_prompt));
    app.type_text("hunter2");
    assert_eq!(
        app.text_of(role(Role::Text).text("typed")),
        "typed: hunter2"
    );
}

#[test]
fn a_field_mounted_outside_an_event_is_focused_by_the_next_frame() {
    let mut app = TestApp::new(component(|cx| {
        let open = cx.signal(false);
        cx.request_animation_frame(move |cx, _| open.set(cx, true));
        show(open, || component(password_prompt), || vstack(()))
    }));
    app.advance_ms(16);
    app.type_text("hunter2");
    assert_eq!(
        app.text_of(role(Role::Text).text("typed")),
        "typed: hunter2"
    );
}
