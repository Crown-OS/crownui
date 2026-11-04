use crowntest::{Role, TestApp, role, text as text_query};
use view::{Listeners, Styled, bind, component};
use widgets::{
    kit::{
        ButtonOptions, ButtonVariant, SidebarItemOptions, Theme, icon_button, icons, provide_theme,
        sidebar_item,
    },
    text, vstack,
};

#[test]
fn a_trailing_caption_sits_at_the_rows_end_in_twelve_pixel_text() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        vstack(sidebar_item("Downloads", icons::DOWNLOAD, false).trailing_text("3"))
            .w(240.0)
            .p(8.0)
    }));
    let row = app.find(role(Role::MenuItem).name("Downloads 3")).bounds;
    let layout = app.layout_snapshot();
    let caption = format!("stack[4] > text @{},", row.max_x() - 10.0 - 7.2);
    assert!(layout.contains(&caption), "{caption} in {layout}");
    assert!(layout.contains(" 7.2x14.4"), "{layout}");
}

#[test]
fn a_trailing_view_follows_the_caption_and_takes_its_own_clicks() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let ejected = cx.signal(false);
        vstack((
            sidebar_item("USB Drive", icons::HARD_DRIVE, false)
                .trailing_text("32 GB")
                .trailing(
                    icon_button(icons::ARROW_UP_FROM_LINE, "Eject")
                        .variant(ButtonVariant::Ghost)
                        .on_click(move |cx, event| {
                            event.stop_propagation();
                            ejected.set(cx, true);
                        }),
                ),
            text(bind(move |rt| {
                if ejected.get(rt) {
                    "ejected".to_owned()
                } else {
                    "mounted".to_owned()
                }
            })),
        ))
        .w(260.0)
        .p(8.0)
    }));
    let row = app
        .find(role(Role::MenuItem).name("USB Drive 32 GB Eject"))
        .bounds;
    let (x, y) = (row.max_x() - 10.0 - 14.0, row.center().y);
    app.pointer_down(x, y).pointer_up(x, y);
    assert!(app.exists(text_query("ejected")));
    insta::assert_snapshot!(app.snapshot());
}

#[test]
fn rows_without_trailing_content_keep_their_layout() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        vstack(sidebar_item("Home", icons::HOUSE, true)).w(240.0)
    }));
    assert!(app.exists(role(Role::MenuItem).name("Home")));
    let layout = app.layout_snapshot();
    assert!(
        layout.contains("kit_sidebar_item > text[3] @38,"),
        "{layout}"
    );
}
