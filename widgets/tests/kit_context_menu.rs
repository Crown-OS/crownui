use crowntest::{Role, TestApp, role};
use view::{Listeners, Styled, component};
use widgets::{
    kit::{
        MenuRowOptions, Theme, button, context_menu, icons, menu_header, menu_item, menu_separator,
        menu_submenu, preview_menu, provide_theme,
    },
    vstack,
};

fn menu_app(theme: Theme) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, theme);
        vstack(preview_menu()).p(24.0)
    }))
}

#[test]
fn the_menu_is_340_wide_and_rows_are_30_tall_inside_a_one_pixel_border() {
    let mut app = menu_app(Theme::dark());
    let menu = app.find(role(Role::Menu)).bounds;
    assert_eq!(menu.size.width, 340.0);
    assert_eq!(menu.origin.x, 24.0);

    let undo = app.find(role(Role::MenuItem).name("Undo")).bounds;
    assert_eq!(undo.size.height, 30.0);
    assert_eq!(undo.size.width, 338.0);
    assert_eq!(undo.origin.x, menu.origin.x + 1.0);
    assert_eq!(undo.origin.y, menu.origin.y + 7.0);

    let redo = app.find(role(Role::MenuItem).name("Redo")).bounds;
    assert_eq!(redo.origin.y - undo.origin.y, 32.0);
}

#[test]
fn separators_are_one_pixel_and_headers_twenty_two_with_two_pixel_gaps() {
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
    assert_eq!(separator.origin.y, first.origin.y + 30.0 + 2.0);
    let second = app.find(role(Role::MenuItem).name("Second")).bounds;
    assert_eq!(second.origin.y, separator.origin.y + 1.0 + 2.0 + 22.0 + 2.0);
    assert_eq!(
        app.find(role(Role::Text).name("Section"))
            .bounds
            .size
            .height,
        22.0
    );
}

#[test]
fn the_selected_row_is_filled_with_the_menu_selection_color_and_rounded_to_six() {
    let theme = Theme::dark();
    let app = menu_app(theme);
    let frame = app.frame_snapshot();
    let selected = theme.menu.selected_bg;
    let hex = format!(
        "{:02x}{:02x}{:02x}{:02x}",
        (selected.r * 255.0).round() as u8,
        (selected.g * 255.0).round() as u8,
        (selected.b * 255.0).round() as u8,
        (selected.a * 255.0).round() as u8
    );
    assert!(frame.contains(&format!("r6 #{hex}")), "{frame}");
    assert!(frame.contains("r14"), "{frame}");
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
    let hex = |color: geometry::Color| {
        let channel = |value: f32| (value * 255.0).round() as u8;
        format!(
            "{:02x}{:02x}{:02x}",
            channel(color.r),
            channel(color.g),
            channel(color.b)
        )
    };
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
