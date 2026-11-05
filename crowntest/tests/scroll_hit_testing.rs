use crowntest::{TestApp, role};
use semantics::Role;
use view::{Listeners, Semantic, Styled, bind, component};
use widgets::{button, scroll_view, text, vstack};

const ROW_HEIGHT: f32 = 20.0;
const VIEWPORT_HEIGHT: f32 = 100.0;

/// Fifty button rows in a five-row-tall scroll view; the text below reports the last row
/// clicked and the last row the pointer entered.
fn list_of_buttons() -> TestApp {
    TestApp::with_size(
        component(|cx| {
            let clicked = cx.signal(None::<usize>);
            let entered = cx.signal(None::<usize>);
            let rows: Vec<_> = (0..50)
                .map(|index| {
                    button(format!("Row {index}"))
                        .h(ROW_HEIGHT)
                        .on_click(move |cx, _| clicked.set(cx, Some(index)))
                        .on_pointer_enter(move |cx, _| entered.set(cx, Some(index)))
                })
                .collect();
            vstack((
                scroll_view(vstack(rows))
                    .h(VIEWPORT_HEIGHT)
                    .role(Role::ScrollView),
                text(bind(move |runtime| {
                    let report = |label: &str, value: Option<usize>| {
                        value
                            .map_or_else(|| format!("{label} none"), |row| format!("{label} {row}"))
                    };
                    format!(
                        "{} / {}",
                        report("clicked", clicked.get(runtime)),
                        report("entered", entered.get(runtime))
                    )
                })),
            ))
        }),
        300.0,
        200.0,
    )
}

#[test]
fn a_click_after_scrolling_lands_on_the_row_that_moved_under_the_pointer() {
    let mut app = list_of_buttons();
    app.pointer_down(50.0, 10.0).pointer_up(50.0, 10.0);
    assert_eq!(app.text_of(role(Role::Text)), "clicked 0 / entered none");

    app.scroll(role(Role::ScrollView), 5.0 * ROW_HEIGHT);
    app.pointer_down(50.0, 10.0).pointer_up(50.0, 10.0);
    assert_eq!(app.text_of(role(Role::Text)), "clicked 5 / entered none");

    let bounds = app.find(role(Role::Button).name("Row 5")).bounds;
    assert!(
        bounds.min_y().abs() < 0.5,
        "the fifth row should sit at the top after scrolling five rows: {bounds:?}"
    );
}

#[test]
fn hovering_after_scrolling_follows_the_moved_rows() {
    let mut app = list_of_buttons();
    app.scroll(role(Role::ScrollView), 10.0 * ROW_HEIGHT);
    app.pointer_move(50.0, 30.0);
    assert_eq!(app.text_of(role(Role::Text)), "clicked none / entered 11");
}
