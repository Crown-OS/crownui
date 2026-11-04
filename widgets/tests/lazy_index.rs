use crowntest::{Role, TestApp, name as text_query, role};
use geometry::{Size, Vector};
use view::{Listeners, ScrollHandle, Styled, View, bind, component};
use widgets::{
    LazyListOptions, ScrollOptions, button, hstack, lazy_grid, lazy_list, scroll_view, text, vstack,
};

const ROW_HEIGHT: f32 = 30.0;

fn row_count(app: &mut TestApp) -> usize {
    app.count(role(Role::Text)) - 1
}

fn files(count: usize) -> impl View {
    component(move |cx| {
        let total = cx.signal(count);
        let revision = cx.signal(0_u64);
        let suffix = cx.store(String::new());
        let handle = ScrollHandle::new(cx);
        let rows = lazy_list(total, revision, ROW_HEIGHT, move |cx, index| {
            let label = suffix.with(cx, |suffix| format!("file {index}{suffix}"));
            text(label)
        })
        .scroll_handle(handle)
        .flex_1();
        let controls = hstack((
            button("rename").on_click(move |cx, _| {
                suffix.update(cx, |suffix| suffix.push_str(".bak"));
                revision.update(cx, |revision| *revision += 1);
            }),
            button("reveal").on_click(move |cx, _| handle.ensure_visible(cx, 500)),
            button("top").on_click(move |cx, _| handle.scroll_to_index(cx, 0)),
            button("shrink").on_click(move |cx, _| total.set(cx, 3)),
            text(bind(move |rt| {
                let range = handle.visible_range(rt);
                format!("visible {}..{}", range.start, range.end)
            })),
        ))
        .h(40.0);
        vstack((controls, rows)).h_full()
    })
}

#[test]
fn a_list_of_a_hundred_thousand_rows_mounts_only_the_screenful() {
    let mut app = TestApp::with_size(files(100_000), 400.0, 340.0);
    assert!(app.exists(text_query("file 0")));
    assert_eq!(row_count(&mut app), 10 + 4);
    assert!(
        app.text_of(role(Role::Text).name("visible 0..10"))
            .contains("0..10")
    );
}

#[test]
fn scrolling_mounts_the_rows_that_come_into_view() {
    let mut app = TestApp::with_size(files(100_000), 400.0, 340.0);
    app.scroll(text_query("file 0"), 3000.0);
    assert!(app.exists(text_query("file 100")));
    assert!(!app.exists(text_query("file 0")));
    let row = app.find(text_query("file 100")).bounds;
    assert_eq!(row.origin.y, 40.0);
    assert!(app.exists(role(Role::Text).name("visible 100..110")));
}

#[test]
fn bumping_the_revision_builds_the_rows_in_view_again() {
    let mut app = TestApp::with_size(files(1000), 400.0, 340.0);
    app.click(role(Role::Button).name("rename"));
    assert!(app.exists(text_query("file 3.bak")));
    assert!(!app.exists(text_query("file 3")));
}

#[test]
fn the_handle_scrolls_an_index_into_view_and_back_to_the_top() {
    let mut app = TestApp::with_size(files(1000), 400.0, 340.0);
    app.click(role(Role::Button).name("reveal"));
    let row = app.find(text_query("file 500")).bounds;
    assert_eq!(row.max_y(), 340.0);
    app.click(role(Role::Button).name("top"));
    assert_eq!(app.find(text_query("file 0")).bounds.origin.y, 40.0);
}

#[test]
fn a_list_that_shrinks_under_the_scroll_position_returns_to_its_content() {
    let mut app = TestApp::with_size(files(1000), 400.0, 340.0);
    app.click(role(Role::Button).name("reveal"));
    app.click(role(Role::Button).name("shrink"));
    app.settle();
    assert_eq!(row_count(&mut app), 3);
    assert_eq!(app.find(text_query("file 0")).bounds.origin.y, 40.0);
}

fn icons(count: usize) -> impl View {
    component(move |cx| {
        let total = cx.signal(count);
        let revision = cx.signal(0_u64);
        let handle = ScrollHandle::new(cx);
        let columns = handle.columns();
        vstack((
            text(bind(move |rt| format!("columns {}", columns.get(rt)))),
            lazy_grid(
                total,
                revision,
                Size::new(90.0, 100.0),
                (6.0, 2.0),
                |_, index| text(format!("icon {index}")),
            )
            .content_padding(12.0)
            .scroll_handle(handle)
            .flex_1(),
        ))
        .h_full()
    })
}

#[test]
fn a_grid_fits_as_many_columns_as_auto_fill_and_shares_the_spare_width() {
    let mut app = TestApp::with_size(icons(10_000), 800.0, 600.0);
    assert!(app.exists(text_query("columns 8")));
    let first = app.find(text_query("icon 0")).bounds;
    let second = app.find(text_query("icon 1")).bounds;
    let below = app.find(text_query("icon 8")).bounds;
    let width = (800.0 - 24.0 - 7.0 * 6.0) / 8.0;
    assert_eq!(first.origin.x, 12.0);
    assert!(
        (second.origin.x - (12.0 + width + 6.0)).abs() < 0.01,
        "{second:?}"
    );
    assert!((first.size.width - width).abs() < 0.01, "{first:?}");
    assert_eq!(below.origin.y - first.origin.y, 102.0);
}

#[test]
fn a_grid_mounts_whole_rows_in_view_plus_overscan() {
    let mut app = TestApp::with_size(icons(10_000), 800.0, 600.0);
    let visible_rows = 6;
    assert_eq!(app.count(role(Role::Text)), 1 + (visible_rows + 4) * 8);
    app.resize(400, 600);
    app.settle();
    assert!(app.exists(text_query("columns 3")));
    assert_eq!(app.count(role(Role::Text)), 1 + (visible_rows + 4) * 3);
}

#[test]
fn a_scroll_view_follows_a_handle_driven_from_app_code() {
    let mut app = TestApp::with_size(
        component(|cx| {
            let handle = ScrollHandle::new(cx);
            let offset = handle.offset();
            vstack((
                button("down").on_click(move |cx, _| {
                    handle.scroll_to_offset(cx, Vector::new(0.0, 10_000.0));
                }),
                text(bind(move |rt| format!("at {}", offset.get(rt).y))),
                scroll_view(vstack(text("tall").h(1000.0)))
                    .scroll_handle(handle)
                    .h(200.0),
            ))
        }),
        400.0,
        400.0,
    );
    app.click(role(Role::Button).name("down"));
    assert!(app.exists(text_query("at 800")));
    let tall = app.find(text_query("tall")).bounds;
    let viewport = app.find(role(Role::ScrollView)).bounds;
    assert!(
        (tall.max_y() - viewport.max_y()).abs() < 0.01,
        "{tall:?} {viewport:?}"
    );
    app.scroll(text_query("tall"), -300.0);
    assert!(app.exists(text_query("at 500")));
}
