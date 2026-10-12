use crowntest::{Role, TestApp, role};
use geometry::Rect;
use view::{AppIconRequest, Listeners, View, bind, component};
use widgets::{button, text, vstack};
use windowing::{AppIconPlacement, Capabilities, WindowEvent};

const APP: &str = "org.example.Editor";
const RADIUS: f32 = 11.0;

fn icon_rect(x: f32) -> Rect {
    Rect::from_xywh(x, 8.0, 52.0, 52.0)
}

fn launcher() -> impl View {
    component(|cx| {
        let x = cx.signal(10.0_f32);
        let status = cx.signal(String::from("idle"));
        cx.on_launch_app(move |cx, app_id| status.set(cx, format!("launch {app_id}")));
        vstack((
            text(bind(move |rt| status.get(rt))),
            button("show").on_click(move |cx, _| {
                let at = x.get(cx);
                cx.set_app_icon(APP, icon_rect(at), RADIUS);
            }),
            button("move").on_click(move |cx, _| x.set(cx, 70.0)),
            button("hide").on_click(|cx, _| cx.remove_app_icon(APP)),
            button("open").on_click(move |cx, _| {
                if !cx.activate_app_icon(APP) {
                    status.set(cx, "launched directly".into());
                }
            }),
        ))
    })
}

fn supporting_app_icons(app: &mut TestApp) {
    app.ui().set_platform_capabilities(Capabilities {
        app_icons: true,
        ..Capabilities::default()
    });
}

fn click(app: &mut TestApp, name: &str) {
    app.click(role(Role::Button).name(name));
}

fn status(app: &mut TestApp) -> String {
    app.text_of(role(Role::Text))
}

#[test]
fn an_icon_is_sent_once_per_change_and_removed_once() {
    let mut app = TestApp::new(launcher());
    click(&mut app, "show");
    click(&mut app, "show");
    let placement = AppIconPlacement {
        rect: icon_rect(10.0),
        corner_radius: RADIUS,
    };
    assert_eq!(
        app.ui().take_app_icon_requests(),
        [AppIconRequest::Set(APP.into(), placement)]
    );
    click(&mut app, "move");
    click(&mut app, "show");
    click(&mut app, "hide");
    click(&mut app, "hide");
    let requests = app.ui().take_app_icon_requests();
    assert_eq!(requests.len(), 2);
    assert!(matches!(&requests[0], AppIconRequest::Set(_, moved) if moved.rect == icon_rect(70.0)));
    assert_eq!(requests[1], AppIconRequest::Remove(APP.into()));
}

#[test]
fn without_the_capability_a_click_launches_directly() {
    let mut app = TestApp::new(launcher());
    click(&mut app, "show");
    click(&mut app, "open");
    assert_eq!(status(&mut app), "launched directly");
}

#[test]
fn an_unshown_icon_cannot_be_activated() {
    let mut app = TestApp::new(launcher());
    supporting_app_icons(&mut app);
    click(&mut app, "open");
    assert_eq!(status(&mut app), "launched directly");
    assert!(app.ui().take_app_icon_requests().is_empty());
}

#[test]
fn an_activated_icon_goes_to_the_compositor_which_may_ask_for_a_launch() {
    let mut app = TestApp::new(launcher());
    supporting_app_icons(&mut app);
    click(&mut app, "show");
    click(&mut app, "open");
    let requests = app.ui().take_app_icon_requests();
    assert_eq!(requests.last(), Some(&AppIconRequest::Activate(APP.into())));
    assert_eq!(status(&mut app), "idle");
    if let Err(error) = app.inline().handle(&WindowEvent::LaunchApp(APP.into())) {
        panic!("frame failed: {error}");
    }
    assert_eq!(status(&mut app), format!("launch {APP}"));
}
