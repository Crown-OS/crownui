use crowntest::{Role, TestApp, role};
use view::{Listeners, component, show};
use widgets::{button, text as label, vstack};

#[test]
fn a_window_without_listeners_closes_when_asked() {
    let mut app = TestApp::new(label("Idle"));
    assert!(app.request_close());
}

#[test]
fn a_busy_app_keeps_its_window_and_closes_once_the_user_confirms() {
    let mut app = TestApp::new(component(|cx| {
        let asking = cx.signal(false);
        cx.on_close_requested(move |cx, event| {
            event.prevent_default();
            asking.set(cx, true);
        });
        vstack((
            label("Writing image…"),
            show(
                asking,
                move || button("Quit anyway").on_click(|cx, _| cx.close_window()),
                || vstack(()),
            ),
        ))
    }));
    assert!(!app.request_close());
    assert!(!app.take_close_request());
    app.click(role(Role::Button).name("Quit anyway"));
    assert!(app.take_close_request());
}

#[test]
fn an_unmounted_guard_no_longer_holds_the_window() {
    let mut app = TestApp::new(component(|cx| {
        let busy = cx.signal(true);
        vstack((
            show(
                busy,
                || {
                    component(|cx| {
                        cx.on_close_requested(|_, event| event.prevent_default());
                        label("Busy")
                    })
                },
                || vstack(()),
            ),
            button("Finish").on_click(move |cx, _| busy.set(cx, false)),
        ))
    }));
    assert!(!app.request_close());
    app.click(role(Role::Button).name("Finish"));
    assert!(app.request_close());
}
