use crowntest::{Role, TestApp, role, text as text_query};
use view::{Styled, View, bind, component};
use widgets::{
    hstack,
    kit::{PathBarOptions, PathSegment, Theme, icons, path_bar, provide_theme},
    text, vstack,
};
use windowing::{DndAction, DndActions};

const URI_LIST: &str = "text/uri-list";

fn segments() -> Vec<PathSegment> {
    ["Documents", "Projects", "crownos"].into_iter().fold(
        vec![PathSegment::new("Home").icon(icons::HOUSE)],
        |mut path, name| {
            path.push(PathSegment::new(name));
            path
        },
    )
}

fn browser(width: f32) -> impl View {
    component(move |cx| {
        provide_theme(cx, Theme::dark());
        let path = cx.signal(segments());
        let status = cx.signal(String::from("idle"));
        let bar = path_bar(path, move |cx, index| {
            path.update(cx, |path| path.truncate(index + 1));
            status.set(cx, format!("open {index}"));
        })
        .on_segment_drag_over(move |_, index, event| {
            if index < 2 && event.offers(URI_LIST) {
                event.accept_with(URI_LIST, DndAction::Move);
            }
        })
        .on_segment_drop(move |cx, index, event| {
            let data = String::from_utf8_lossy(&event.data).into_owned();
            status.set(cx, format!("drop {data} on {index} as {:?}", event.action));
        });
        vstack((
            hstack(bar).w(width).h(32.0),
            text(bind(move |rt| status.get(rt))),
        ))
        .p(16.0)
        .items_start()
    })
}

fn shows(app: &mut TestApp, status: &str) -> bool {
    app.exists(text_query(status))
}

#[test]
fn every_segment_is_a_pill_after_a_chevron_and_the_last_is_current() {
    let mut app = TestApp::new(browser(600.0));
    app.settle();
    assert_eq!(app.count(role(Role::Button)), 4);
    let home = app.find(role(Role::Button).name("Home")).bounds;
    let documents = app.find(role(Role::Button).name("Documents")).bounds;
    assert_eq!(home.origin.x, 16.0);
    assert!(
        (home.size.height - (4.0 + 16.8 + 4.0)).abs() < 0.01,
        "{home:?}"
    );
    assert!(
        documents.origin.x > home.max_x() + 14.0,
        "{home:?} {documents:?}"
    );
    insta::assert_snapshot!(app.snapshot());
}

#[test]
fn clicking_a_segment_selects_its_index() {
    let mut app = TestApp::new(browser(600.0));
    app.settle();
    app.click(role(Role::Button).name("Documents"));
    assert!(shows(&mut app, "open 1"));
    app.settle();
    assert_eq!(app.count(role(Role::Button)), 2);
}

#[test]
fn a_path_too_long_for_its_row_folds_its_leading_segments() {
    let mut app = TestApp::new(browser(220.0));
    app.settle();
    assert!(!app.exists(role(Role::Button).name("Home")));
    assert!(app.exists(role(Role::Button).name("crownos")));
    let shown = app.find(role(Role::Button).name("crownos")).bounds;
    assert!(shown.max_x() <= 16.0 + 220.0, "{shown:?}");
    let folded = app.count(role(Role::Button)) < 4;
    assert!(folded);
    app.click(role(Role::Button).name("More"));
    assert!((0..3).any(|index| shows(&mut app, &format!("open {index}"))));
}

#[test]
fn a_segment_takes_a_drop_its_handler_accepts() {
    let mut app = TestApp::new(browser(600.0));
    app.settle();
    let documents = app
        .find(role(Role::Button).name("Documents"))
        .bounds
        .center();
    app.drag_enter(documents.x, documents.y, &[URI_LIST], DndActions::COPY_MOVE);
    let accepted = app
        .drag_acceptance()
        .flatten()
        .map(|accepted| accepted.action);
    assert_eq!(accepted, Some(DndAction::Move));
    app.drop_data(
        documents.x,
        documents.y,
        URI_LIST,
        b"file:///a",
        DndAction::Move,
    );
    assert!(shows(&mut app, "drop file:///a on 1 as Move"));
}

#[test]
fn a_segment_whose_handler_refuses_stays_unlit() {
    let mut app = TestApp::new(browser(600.0));
    app.settle();
    let current = app.find(role(Role::Button).name("crownos")).bounds.center();
    app.drag_enter(current.x, current.y, &[URI_LIST], DndActions::COPY_MOVE);
    assert_eq!(app.drag_acceptance(), None);
    app.drag_leave();
    assert!(shows(&mut app, "idle"));
}
