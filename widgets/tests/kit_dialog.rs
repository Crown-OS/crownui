use crowntest::{Role, TestApp, role, text as text_query};
use view::{Listeners, Styled, component};
use widgets::{
    kit::{DialogOptions, Theme, button, dialog, provide_theme},
    text, vstack,
};

fn confirm(dismiss_on_scrim: bool) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, Theme::dark());
        let open = cx.signal(false);
        vstack((
            button("Open").on_click(move |cx, _| open.set(cx, true)),
            vstack(
                dialog(open, move || {
                    (
                        text("Delete 3 items?"),
                        button("Cancel").on_click(move |cx, _| open.set(cx, false)),
                        button("Delete"),
                    )
                })
                .dismiss_on_scrim(dismiss_on_scrim),
            )
            .p(40.0),
        ))
        .p(24.0)
        .items_start()
    }))
}

fn opened(dismiss_on_scrim: bool) -> TestApp {
    let mut app = confirm(dismiss_on_scrim);
    app.click(role(Role::Button).name("Open"));
    app.settle();
    app
}

fn focused_name(app: &mut TestApp) -> Option<String> {
    let id = app.ui().focused_semantic_id()?;
    app.ui().semantic_tree().get(id)?.name.clone()
}

#[test]
fn a_closed_dialog_shows_nothing() {
    let mut app = confirm(true);
    assert!(!app.exists(role(Role::Dialog)));
}

#[test]
fn an_open_dialog_centers_its_card_over_a_scrim_covering_the_window() {
    let mut app = opened(true);
    let card = app.find(role(Role::Dialog)).bounds;
    assert!((card.center().x - 400.0).abs() < 0.5, "{card:?}");
    assert!((card.center().y - 300.0).abs() < 0.5, "{card:?}");
    assert_eq!(card.size.width, 320.0);
    let layout = app.layout_snapshot();
    assert!(layout.contains("kit_dialog_sheet @0,0 800x600"), "{layout}");
}

#[test]
fn focus_moves_into_the_card_and_tab_cycles_inside_it() {
    let mut app = opened(true);
    assert_eq!(focused_name(&mut app).as_deref(), Some("Cancel"));
    app.press_key("Tab");
    assert_eq!(focused_name(&mut app).as_deref(), Some("Delete"));
    app.press_key("Tab");
    assert_eq!(focused_name(&mut app).as_deref(), Some("Cancel"));
    app.press_key("Shift+Tab");
    assert_eq!(focused_name(&mut app).as_deref(), Some("Delete"));
}

#[test]
fn escape_closes_the_dialog() {
    let mut app = opened(true);
    app.press_key("Escape").settle();
    assert!(!app.exists(role(Role::Dialog)));
}

#[test]
fn pressing_the_scrim_closes_it_unless_turned_off() {
    let mut app = opened(true);
    app.pointer_down(780.0, 20.0)
        .pointer_up(780.0, 20.0)
        .settle();
    assert!(!app.exists(role(Role::Dialog)));

    let mut kept = opened(false);
    kept.pointer_down(780.0, 20.0)
        .pointer_up(780.0, 20.0)
        .settle();
    assert!(kept.exists(role(Role::Dialog)));
}

#[test]
fn the_scrim_keeps_presses_from_the_content_behind_it() {
    let mut app = opened(false);
    let open = app.find(role(Role::Button).name("Open")).bounds.center();
    app.pointer_down(open.x, open.y).pointer_up(open.x, open.y);
    assert!(app.exists(text_query("Delete 3 items?")));
    assert_eq!(focused_name(&mut app).as_deref(), Some("Cancel"));
}

#[test]
fn a_button_inside_can_close_it() {
    let mut app = opened(true);
    app.click(role(Role::Button).name("Cancel")).settle();
    assert!(!app.exists(role(Role::Dialog)));
}
