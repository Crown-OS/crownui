use crowntest::{Role, TestApp, role, text as text_query};
use geometry::Point;
use view::{Listeners, Styled, bind, component};
use widgets::{
    kit::{Theme, button, context_menu_at, menu_item, provide_theme},
    text, vstack,
};

fn files(at: Point) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, Theme::dark());
        let position = cx.signal(None);
        let picked = cx.signal(String::from("none"));
        vstack((
            button("Open menu").on_click(move |cx, _| position.set(cx, Some(at))),
            text(bind(move |rt| format!("picked {}", picked.get(rt)))),
            vstack(context_menu_at(position, move || {
                (
                    menu_item("Rename").on_click(move |cx, _| picked.set(cx, "rename".into())),
                    menu_item("Move to Trash"),
                )
            }))
            .p(32.0),
        ))
        .p(24.0)
        .items_start()
    }))
}

fn opened(at: Point) -> TestApp {
    let mut app = files(at);
    app.click(role(Role::Button).name("Open menu"));
    app.settle();
    app
}

#[test]
fn the_menu_opens_right_of_and_below_the_pointer() {
    let mut app = opened(Point::new(300.0, 200.0));
    let menu = app.find(role(Role::Menu)).bounds;
    assert_eq!(menu.origin, Point::new(300.0, 200.0));
    assert_eq!(menu.size.width, 340.0);
}

#[test]
fn near_the_window_corner_the_menu_flips_left_and_up() {
    let mut app = opened(Point::new(700.0, 560.0));
    let menu = app.find(role(Role::Menu)).bounds;
    assert_eq!(menu.max_x(), 700.0, "{menu:?}");
    assert_eq!(menu.max_y(), 560.0, "{menu:?}");
}

#[test]
fn picking_a_row_runs_it_and_closes_the_menu() {
    let mut app = opened(Point::new(300.0, 200.0));
    app.click(role(Role::MenuItem).name("Rename")).settle();
    assert!(app.exists(text_query("picked rename")));
    assert!(!app.exists(role(Role::Menu)));
}

#[test]
fn escape_closes_the_menu() {
    let mut app = opened(Point::new(300.0, 200.0));
    app.press_key("Escape").settle();
    assert!(!app.exists(role(Role::Menu)));
}

#[test]
fn a_press_outside_closes_the_menu_without_reaching_what_is_under_it() {
    let mut app = opened(Point::new(300.0, 200.0));
    let open = app
        .find(role(Role::Button).name("Open menu"))
        .bounds
        .center();
    app.pointer_down(open.x, open.y)
        .pointer_up(open.x, open.y)
        .settle();
    assert!(!app.exists(role(Role::Menu)));
    let inside = Point::new(310.0, 215.0);
    let mut kept = opened(Point::new(300.0, 200.0));
    kept.pointer_down(inside.x, inside.y).settle();
    assert!(kept.exists(role(Role::Menu)));
}
