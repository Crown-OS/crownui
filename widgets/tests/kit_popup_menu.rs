use crowntest::{Role, TestApp, role, text as text_query};
use view::{Listeners, Semantic, Styled, bind, component};
use widgets::{
    button,
    kit::{Theme, icons, popup_menu, popup_menu_item, popup_menu_separator, provide_theme},
    spacer, text, vstack,
};

const TRIGGER: &str = "Network options";

fn menu_row(app: &mut TestApp, name: &str) -> geometry::Rect {
    app.find(role(Role::MenuItem).name(name)).bounds
}

fn network_menu(app_theme: Theme) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, app_theme);
        let picked = cx.signal(String::new());
        let auto_join = cx.signal(true);
        vstack((
            vstack((
                spacer(),
                popup_menu(
                    icons::ELLIPSIS,
                    TRIGGER,
                    vec![
                        popup_menu_item("Auto-Join", move |cx| {
                            auto_join.update(cx, |on| *on = !*on);
                            picked.set(cx, "auto-join".into());
                        })
                        .checked(auto_join),
                        popup_menu_item("Network Settings…", move |cx| {
                            picked.set(cx, "settings".into())
                        }),
                        popup_menu_separator(),
                        popup_menu_item("Forget This Network…", move |cx| {
                            picked.set(cx, "forget".into())
                        })
                        .destructive(),
                        popup_menu_item("Unavailable", move |cx| {
                            picked.set(cx, "unavailable".into())
                        })
                        .disabled(true),
                    ],
                ),
            ))
            .flex_row()
            .w(400.0),
            button("Below").on_click(move |cx, _| picked.set(cx, "below".into())),
            text(bind(move |runtime| {
                format!("Picked {}", picked.get(runtime))
            })),
        ))
        .p(24.0)
        .gap(12.0)
    }))
}

fn open(app: &mut TestApp) {
    app.click(role(Role::Button).name(TRIGGER));
    app.settle();
}

#[test]
fn the_trigger_is_a_small_icon_pill_and_the_closed_menu_has_no_panel() {
    let mut app = network_menu(Theme::dark());
    let trigger = app.find(role(Role::Button).name(TRIGGER)).bounds;
    assert_eq!((trigger.size.width, trigger.size.height), (28.0, 22.0));
    assert!(!app.exists(role(Role::Menu)));
}

#[test]
fn opening_hangs_the_panel_below_the_trigger_right_aligned_without_moving_siblings() {
    let mut app = network_menu(Theme::dark());
    let sibling_before = app.find(role(Role::Button).name("Below")).bounds;
    let trigger = app.find(role(Role::Button).name(TRIGGER)).bounds;

    open(&mut app);
    let panel = app.find(role(Role::Menu)).bounds;
    assert_eq!(panel.max_x(), trigger.max_x());
    assert_eq!(panel.origin.y, trigger.max_y() + 5.0);
    assert!((168.0..=320.0).contains(&panel.size.width), "{panel:?}");
    assert_eq!(panel.size.height, 10.0 + 4.0 * 28.0 + 7.0);
    assert_eq!(
        app.find(role(Role::Button).name("Below")).bounds,
        sibling_before
    );

    let first = menu_row(&mut app, "Auto-Join");
    assert_eq!(first.size.height, 28.0);
    assert_eq!(first.origin.y, panel.origin.y + 5.0);
    assert_eq!(first.origin.x, panel.origin.x + 5.0);
    assert_eq!(first.size.width, panel.size.width - 10.0);
    let settings = menu_row(&mut app, "Network Settings…");
    assert_eq!(settings.origin.y, first.origin.y + 28.0);
    let forget = menu_row(&mut app, "Forget This Network…");
    assert_eq!(forget.origin.y, settings.origin.y + 28.0 + 7.0);
    assert_eq!(app.find(role(Role::Separator)).bounds.size.height, 1.0);
}

#[test]
fn picking_a_row_runs_its_callback_and_closes_the_menu() {
    let mut app = network_menu(Theme::dark());
    open(&mut app);
    app.click(role(Role::MenuItem).name("Network Settings…"));
    assert_eq!(app.text_of(text_query("Picked")), "Picked settings");
    app.settle();
    assert!(!app.exists(role(Role::Menu)));
}

#[test]
fn a_checked_row_reports_its_state_and_follows_its_signal() {
    let mut app = network_menu(Theme::dark());
    open(&mut app);
    assert_eq!(
        app.find(role(Role::MenuItem).name("Auto-Join"))
            .states
            .checked,
        Some(true)
    );
    assert_eq!(
        app.find(role(Role::MenuItem).name("Network Settings…"))
            .states
            .checked,
        None
    );
    app.click(role(Role::MenuItem).name("Auto-Join"));
    app.settle();
    open(&mut app);
    assert_eq!(
        app.find(role(Role::MenuItem).name("Auto-Join"))
            .states
            .checked,
        Some(false)
    );
}

#[test]
fn a_disabled_row_cannot_be_picked_and_a_separator_is_not_a_row() {
    let mut app = network_menu(Theme::dark());
    open(&mut app);
    let unavailable = app.find(role(Role::MenuItem).name("Unavailable"));
    assert!(unavailable.states.disabled);
    let center = unavailable.bounds.center();
    app.pointer_down(center.x, center.y);
    app.pointer_up(center.x, center.y);
    assert_eq!(app.text_of(text_query("Picked")), "Picked ");
    assert!(app.exists(role(Role::Menu)));

    let separator = app.find(role(Role::Separator)).bounds.center();
    app.pointer_down(separator.x, separator.y);
    app.pointer_up(separator.x, separator.y);
    assert!(app.exists(role(Role::Menu)));
}

#[test]
fn clicking_a_row_with_the_pointer_picks_it_without_losing_focus_first() {
    let mut app = network_menu(Theme::dark());
    open(&mut app);
    let center = menu_row(&mut app, "Forget This Network…").center();
    app.pointer_down(center.x, center.y);
    assert!(app.exists(role(Role::Menu)));
    app.pointer_up(center.x, center.y);
    assert_eq!(app.text_of(text_query("Picked")), "Picked forget");
}

#[test]
fn pressing_outside_the_panel_or_on_the_trigger_again_closes_it() {
    let mut app = network_menu(Theme::dark());
    open(&mut app);
    app.pointer_down(700.0, 500.0);
    app.pointer_up(700.0, 500.0);
    app.settle();
    assert!(!app.exists(role(Role::Menu)));

    open(&mut app);
    app.click(role(Role::Button).name(TRIGGER));
    app.settle();
    assert!(!app.exists(role(Role::Menu)));
}

#[test]
fn escape_closes_and_arrow_keys_drive_the_highlight_then_enter_picks() {
    let mut app = network_menu(Theme::dark());
    app.focus(role(Role::Button).name(TRIGGER));
    app.press_key("ArrowDown");
    app.settle();
    assert!(app.exists(role(Role::Menu)));

    app.press_key("ArrowDown");
    app.press_key("ArrowDown");
    app.press_key("ArrowDown");
    app.press_key("ArrowDown");
    app.press_key("Enter");
    assert_eq!(app.text_of(text_query("Picked")), "Picked forget");
    app.settle();
    assert!(!app.exists(role(Role::Menu)));

    app.press_key("ArrowUp");
    app.settle();
    assert!(app.exists(role(Role::Menu)));
    app.press_key("Escape");
    app.settle();
    assert!(!app.exists(role(Role::Menu)));
}

#[test]
fn a_disabled_menu_never_opens() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        vstack(
            popup_menu(
                icons::ELLIPSIS,
                TRIGGER,
                vec![popup_menu_item("Only", |_| {})],
            )
            .disabled(true),
        )
        .p(24.0)
    }));
    assert!(app.find(role(Role::Button).name(TRIGGER)).states.disabled);
    let trigger = app.find(role(Role::Button).name(TRIGGER)).id;
    assert!(
        app.ui()
            .perform(trigger, &semantics::UiAction::Click)
            .is_err()
    );
    assert!(!app.exists(role(Role::Menu)));
}

#[test]
fn the_panel_animates_in_and_the_hovered_row_lights_up_at_once() {
    let mut app = network_menu(Theme::dark());
    app.click(role(Role::Button).name(TRIGGER));
    app.advance_ms(16);
    assert!(app.is_animating());
    assert!(
        app.frame_snapshot().contains("opacity"),
        "{}",
        app.frame_snapshot()
    );
    app.settle();
    assert!(!app.is_animating());

    let selected = Theme::dark().menu.selected_bg;
    let accent_fill = format!(
        "r6 #{:02x}{:02x}{:02x}ff",
        (selected.r * 255.0).round() as u8,
        (selected.g * 255.0).round() as u8,
        (selected.b * 255.0).round() as u8
    );
    assert!(!app.frame_snapshot().contains(&accent_fill));
    let center = menu_row(&mut app, "Network Settings…").center();
    app.pointer_move(center.x, center.y);
    app.settle();
    assert!(
        app.frame_snapshot().contains(&accent_fill),
        "{}",
        app.frame_snapshot()
    );
    app.pointer_move(700.0, 500.0);
    app.settle();
    assert!(!app.frame_snapshot().contains(&accent_fill));
}

#[test]
fn the_open_panel_sits_above_content_that_would_otherwise_cover_it() {
    let mut app = network_menu(Theme::dark());
    open(&mut app);
    let below = app.find(role(Role::Button).name("Below")).bounds;
    let row = menu_row(&mut app, "Network Settings…");
    assert!(row.intersection(below).is_some());
    let covered = row
        .intersection(below)
        .map(|overlap| overlap.center())
        .unwrap_or_default();
    app.pointer_down(covered.x, covered.y);
    app.pointer_up(covered.x, covered.y);
    assert_eq!(app.text_of(text_query("Picked")), "Picked settings");
}

#[test]
fn the_panel_paints_the_menu_palette_in_both_themes() {
    for (name, theme) in [("dark", Theme::dark()), ("light", Theme::light())] {
        let mut app = network_menu(theme);
        open(&mut app);
        insta::with_settings!({ snapshot_suffix => name }, {
            insta::assert_snapshot!("open_menu", app.frame_snapshot());
        });
    }
}

#[test]
fn closing_removes_the_panel_and_restores_the_stacking_it_borrowed() {
    let mut app = network_menu(Theme::dark());
    open(&mut app);
    assert!(app.frame_snapshot().contains("r10"));
    app.press_key("Escape");
    app.settle();
    assert!(!app.frame_snapshot().contains("r10"));
    let below = app.find(role(Role::Button).name("Below")).bounds.center();
    app.pointer_down(below.x, below.y);
    app.pointer_up(below.x, below.y);
    assert_eq!(app.text_of(text_query("Picked")), "Picked below");
}

#[test]
fn a_trigger_near_the_left_edge_keeps_its_panel_inside_the_window() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        vstack(popup_menu(
            icons::ELLIPSIS,
            TRIGGER,
            vec![popup_menu_item("Network Settings…", |_| {})],
        ))
        .p(24.0)
    }));
    open(&mut app);
    let panel = app.find(role(Role::Menu)).bounds;
    assert_eq!(panel.origin.x, 0.0);
    assert!(panel.size.width >= 168.0);
}

#[test]
fn a_menu_near_the_bottom_edge_opens_upward() {
    let mut app = TestApp::with_size(
        component(|cx| {
            provide_theme(cx, Theme::dark());
            vstack((
                spacer(),
                popup_menu(
                    icons::ELLIPSIS,
                    TRIGGER,
                    vec![
                        popup_menu_item("One", |_| {}),
                        popup_menu_item("Two", |_| {}),
                    ],
                ),
            ))
            .p(24.0)
            .h_full()
        }),
        300.0,
        200.0,
    );
    let trigger = app.find(role(Role::Button).name(TRIGGER)).bounds;
    open(&mut app);
    let panel = app.find(role(Role::Menu)).bounds;
    assert_eq!(panel.max_y(), trigger.origin.y - 5.0);
}
