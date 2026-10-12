use crowntest::{Role, TestApp, role};
use geometry::Point;
use view::{Listeners, Styled, View, bind, component};
use widgets::{button, text, vstack};
use windowing::{DragOffer, DragOutcome, DragSource, DroppedData, WindowEvent};

const URI_LIST: &str = "text/uri-list";

fn deliver(app: &mut TestApp, event: WindowEvent) {
    if let Err(error) = app.inline().handle(&event) {
        panic!("frame failed: {error}");
    }
}

fn drop_zone() -> impl View {
    component(|cx| {
        let status = cx.signal(String::from("idle"));
        vstack((
            text(bind(move |rt| status.get(rt))),
            button("drag").on_click(move |cx, _| {
                let source = DragSource {
                    items: vec![(URI_LIST.to_owned(), b"file:///a.desktop".to_vec())],
                    icon: None,
                };
                cx.start_drag(source, move |cx, outcome| {
                    status.set(cx, format!("ended {outcome:?}"));
                });
            }),
        ))
        .w(200.0)
        .h(200.0)
        .on_drag_over(move |cx, event| {
            if event.offers(URI_LIST) {
                event.accept(URI_LIST);
                status.set(cx, "over".into());
            }
        })
        .on_drag_leave(move |cx, _| status.set(cx, "left".into()))
        .on_drop(move |cx, event| {
            status.set(cx, String::from_utf8_lossy(&event.data).into_owned());
        })
    })
}

fn offer(mime_type: &str) -> WindowEvent {
    WindowEvent::DragEnter(DragOffer {
        position: Point::new(20.0, 20.0),
        mime_types: vec![mime_type.to_owned()],
    })
}

fn status(app: &mut TestApp) -> String {
    app.text_of(role(Role::Text))
}

#[test]
fn a_drag_offering_an_accepted_type_is_accepted_once_and_dropped() {
    let mut app = TestApp::new(drop_zone());
    deliver(&mut app, offer(URI_LIST));
    assert_eq!(status(&mut app), "over");
    assert_eq!(
        app.ui().take_drag_acceptance(),
        Some(Some(URI_LIST.to_owned()))
    );
    deliver(&mut app, WindowEvent::DragOver(Point::new(30.0, 30.0)));
    assert_eq!(app.ui().take_drag_acceptance(), None);
    deliver(
        &mut app,
        WindowEvent::Drop(DroppedData {
            position: Point::new(30.0, 30.0),
            mime_type: URI_LIST.to_owned(),
            data: b"file:///b.desktop".to_vec(),
        }),
    );
    assert_eq!(status(&mut app), "file:///b.desktop");
}

#[test]
fn an_unknown_type_is_refused_and_leaving_fires_dragleave() {
    let mut app = TestApp::new(drop_zone());
    deliver(&mut app, offer("image/png"));
    assert_eq!(app.ui().take_drag_acceptance(), None);
    assert_eq!(status(&mut app), "idle");
    deliver(&mut app, offer(URI_LIST));
    deliver(&mut app, WindowEvent::DragLeave);
    assert_eq!(status(&mut app), "left");
}

#[test]
fn a_started_drag_reaches_the_platform_once_and_reports_its_end() {
    let mut app = TestApp::new(drop_zone());
    app.click(role(Role::Button).name("drag"));
    let source = app.ui().take_drag_request();
    assert_eq!(source.map(|source| source.items.len()), Some(1));
    assert!(app.ui().take_drag_request().is_none());
    deliver(&mut app, WindowEvent::DragEnded(DragOutcome::Cancelled));
    assert_eq!(status(&mut app), "ended Cancelled");
}
