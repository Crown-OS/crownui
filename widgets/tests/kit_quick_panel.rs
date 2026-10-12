use crowntest::{Role, TestApp, role};
use view::{Listeners, Styled, View, component};
use widgets::{
    icon,
    kit::{
        PanelHeaderOptions, PanelItemOptions, Theme, panel_action, panel_header, panel_item,
        panel_section, panel_separator, provide_theme, quick_panel,
    },
};

const DOT: &str = r#"<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="8"/></svg>"#;

fn sound() -> TestApp {
    TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let enabled = cx.signal(true);
        let picked = cx.signal(0_u32);
        quick_panel((
            panel_header("Sound").switch(enabled),
            panel_separator(),
            panel_section("Output"),
            panel_item("Speakers")
                .badge(|tint| icon(DOT, tint).size(16.0).boxed())
                .selected(view::bind(move |runtime| picked.get(runtime) == 0))
                .on_click(move |cx, _| picked.set(cx, 0)),
            panel_item("Headphones")
                .badge(|tint| icon(DOT, tint).size(16.0).boxed())
                .detail("70%")
                .selected(view::bind(move |runtime| picked.get(runtime) == 1))
                .on_click(move |cx, _| picked.set(cx, 1)),
            panel_action("Sound Settings…"),
        ))
    }))
}

#[test]
fn rows_keep_the_status_bar_metrics() {
    let mut app = sound();
    let layout = app.layout_snapshot();
    assert!(layout.contains("kit_quick_panel @0,0 296x"), "{layout}");
    assert!(layout.contains("kit_panel_header @0,8 296x36"), "{layout}");
    assert!(
        layout.contains("kit_panel_separator[1] @0,44 296x16"),
        "{layout}"
    );
    assert!(
        layout.contains("kit_panel_section[2] @8,60 280x28"),
        "{layout}"
    );
    assert!(
        layout.contains("kit_panel_item[3] @8,88 280x40"),
        "{layout}"
    );
    assert!(
        layout.contains("kit_panel_action[5] @8,168 280x32"),
        "{layout}"
    );
    assert!(
        layout.contains("kit_panel_separator[1] > stack @16,52 264x1"),
        "{layout}"
    );
}

#[test]
fn the_header_switch_and_the_items_are_operable() {
    let mut app = sound();
    assert_eq!(
        app.find(role(Role::Switch).name("Sound")).states.checked,
        Some(true)
    );
    app.click(role(Role::Switch).name("Sound"));
    assert_eq!(
        app.find(role(Role::Switch).name("Sound")).states.checked,
        Some(false)
    );
    app.click(role(Role::Button).name("Headphones"));
    app.settle();
    assert!(app.exists(role(Role::Button).name("Sound Settings…")));
}

#[test]
fn a_quick_panel_composes_its_rows() {
    let mut app = sound();
    app.settle();
    insta::assert_snapshot!(app.snapshot());
}
