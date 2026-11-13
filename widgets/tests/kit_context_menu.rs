use crowntest::{Role, TestApp, role};
use view::{Listeners, Styled, component};
use widgets::{
    kit::{
        MenuRowOptions, Theme, button, context_menu, icons, menu_header, menu_item,
        menu_item_disabled, menu_separator, menu_submenu, preview_menu, provide_theme,
    },
    vstack,
};

fn menu_app(theme: Theme) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, theme);
        vstack(preview_menu()).p(24.0)
    }))
}

fn hex(color: geometry::Color) -> String {
    color
        .to_rgba8()
        .iter()
        .map(|channel| format!("{channel:02x}"))
        .collect()
}

#[test]
fn rows_are_26_tall_inside_a_6_by_5_inset_and_the_menu_fits_its_widest_row() {
    let mut app = menu_app(Theme::dark());
    let menu = app.find(role(Role::Menu)).bounds;
    assert_eq!(menu.origin.x, 24.0);
    assert!(menu.size.width >= 180.0, "{menu:?}");

    let undo = app.find(role(Role::MenuItem).name("Undo")).bounds;
    assert_eq!(undo.size.height, 26.0);
    assert_eq!(undo.size.width, menu.size.width - 12.0);
    assert_eq!(undo.origin.x, menu.origin.x + 6.0);
    assert_eq!(undo.origin.y, menu.origin.y + 5.0);

    let redo = app.find(role(Role::MenuItem).name("Redo")).bounds;
    assert_eq!(redo.origin.y - undo.origin.y, 26.0);
}

#[test]
fn a_short_menu_keeps_its_minimum_width() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::light());
        vstack(context_menu(menu_item("Open"))).p(24.0)
    }));
    assert_eq!(app.find(role(Role::Menu)).bounds.size.width, 180.0);
}

#[test]
fn separators_are_inset_to_the_labels_inside_a_twelve_pixel_band() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        vstack(context_menu((
            menu_item("First"),
            menu_separator(),
            menu_header("Section"),
            menu_item("Second"),
        )))
        .p(24.0)
    }));
    let first = app.find(role(Role::MenuItem).name("First")).bounds;
    let separator = app.find(role(Role::Separator)).bounds;
    assert_eq!(separator.size.height, 1.0);
    assert_eq!(separator.origin.x, first.origin.x + 12.0);
    assert_eq!(separator.size.width, first.size.width - 24.0);
    assert_eq!(separator.origin.y, first.origin.y + 26.0 + 5.0);
    let second = app.find(role(Role::MenuItem).name("Second")).bounds;
    assert_eq!(second.origin.y, separator.origin.y + 1.0 + 6.0 + 22.0);
    assert_eq!(
        app.find(role(Role::Text).name("Section"))
            .bounds
            .size
            .height,
        22.0
    );
}

#[test]
fn the_selected_row_is_an_accent_capsule_with_a_capsule_rim() {
    let theme = Theme::dark();
    let frame = menu_app(theme).frame_snapshot();
    assert!(
        frame.contains(&format!("r13 #{}", hex(theme.menu.selected_bg))),
        "{frame}"
    );
    assert!(
        frame.contains(&format!("r12.5 #{}", hex(theme.menu.selected_border))),
        "{frame}"
    );
    assert!(frame.contains("r14"), "{frame}");
}

#[test]
fn the_panel_blurs_what_is_behind_it() {
    let frame = menu_app(Theme::light()).frame_snapshot();
    assert!(frame.contains("backdrop-blur30"), "{frame}");
}

#[test]
fn hovering_a_row_lights_it_and_leaving_puts_it_out() {
    let theme = Theme::light();
    let mut app = TestApp::new(component(move |cx| {
        provide_theme(cx, theme);
        vstack(context_menu((menu_item("Open"), menu_item("Rename")))).p(24.0)
    }));
    let lit = format!("r13 #{}", hex(theme.menu.selected_bg));
    assert!(!app.frame_snapshot().contains(&lit));
    app.hover(role(Role::MenuItem).name("Rename")).settle();
    assert_eq!(app.frame_snapshot().matches(&lit).count(), 1);
    app.pointer_move(1.0, 1.0).settle();
    assert!(!app.frame_snapshot().contains(&lit));
}

#[test]
fn a_disabled_row_never_lights_up() {
    let theme = Theme::light();
    let mut app = TestApp::new(component(move |cx| {
        provide_theme(cx, theme);
        vstack(context_menu(menu_item_disabled("Add Link"))).p(24.0)
    }));
    let center = app
        .find(role(Role::MenuItem).name("Add Link"))
        .bounds
        .center();
    app.pointer_move(center.x, center.y).settle();
    let lit = format!("r13 #{}", hex(theme.menu.selected_bg));
    assert!(!app.frame_snapshot().contains(&lit));
}

#[test]
fn disabled_rows_report_disabled_and_carry_the_shortcut_as_description() {
    let mut app = menu_app(Theme::dark());
    let link = app.find(role(Role::MenuItem).name("Add Link"));
    assert!(link.states.disabled);
    assert_eq!(link.description.as_deref(), Some("⌘ K"));
    assert!(!app.find(role(Role::MenuItem).name("Undo")).states.disabled);
}

#[test]
fn a_submenu_row_has_a_chevron_and_no_shortcut() {
    let app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::light());
        vstack((
            context_menu(menu_submenu("Find").leading_icon(icons::FILES)),
            context_menu(menu_item("Find").shortcut_hint("F")),
        ))
        .p(24.0)
        .gap(12.0)
    }));
    let frame = app.frame_snapshot();
    assert_eq!(frame.matches("text \"F\"").count(), 1, "{frame}");
}

#[test]
fn switching_themes_repaints_the_menu_surface() {
    let mut app = TestApp::new(component(|cx| {
        let theme = provide_theme(cx, Theme::light());
        vstack((
            context_menu(menu_item("Paste")),
            button("Dark").on_click(move |cx, _| theme.set(cx, Theme::dark())),
        ))
        .p(24.0)
    }));
    let (light, dark) = (hex(Theme::light().menu.bg), hex(Theme::dark().menu.bg));
    assert!(app.frame_snapshot().contains(&format!("r14 #{light}")));
    app.click(role(Role::Button).name("Dark"));
    assert!(app.frame_snapshot().contains(&format!("r14 #{dark}")));
}

#[test]
fn the_full_reference_menu_paints_as_designed() {
    let app = menu_app(Theme::dark());
    insta::assert_snapshot!(app.frame_snapshot());
}
