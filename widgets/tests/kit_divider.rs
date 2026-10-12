use crowntest::{Role, TestApp, role};
use geometry::rgb;
use view::{Listeners, Styled, component};
use widgets::{
    kit::{DividerOptions, Theme, button, divider, provide_theme},
    text, vstack,
};

fn stage(theme: Theme, content: impl view::ViewSeq) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, theme);
        vstack(content).p(24.0).w(324.0)
    }))
}

#[test]
fn a_divider_is_a_one_pixel_separator_spanning_the_column() {
    let mut app = stage(Theme::dark(), (text("above"), divider(), text("below")));
    let separator = app.find(role(Role::Separator)).bounds;
    assert_eq!((separator.size.width, separator.size.height), (276.0, 1.0));
    assert!(app.frame_snapshot().contains("fill 0,0 276x1 #27272aff"));
}

#[test]
fn the_line_stays_one_pixel_thick_centered_in_a_taller_footprint() {
    let mut app = stage(Theme::dark(), divider().height(12.0));
    let separator = app.find(role(Role::Separator)).bounds;
    assert_eq!(separator.size.height, 12.0);
    let frame = app.frame_snapshot();
    assert!(frame.contains("layer at 0,5.5"), "{frame}");
    assert!(frame.contains("fill 0,0 276x1 #27272aff"), "{frame}");
}

#[test]
fn the_default_color_is_the_surface_border_of_the_theme() {
    let light = stage(Theme::light(), divider());
    assert!(
        light
            .frame_snapshot()
            .contains(&format!("#{:02x}{:02x}{:02x}{:02x}", 0x1b, 0x1b, 0x2a, 18))
    );
}

#[test]
fn a_color_option_replaces_it() {
    let app = stage(Theme::dark(), divider().color(rgb(0xff0000)));
    let frame = app.frame_snapshot();
    assert!(
        frame.contains("#ff0000ff") && !frame.contains("#27272aff"),
        "{frame}"
    );
}

#[test]
fn switching_the_theme_fades_the_line() {
    let mut app = TestApp::new(component(|cx| {
        let theme = provide_theme(cx, Theme::dark());
        vstack((
            divider(),
            button("Light").on_click(move |cx, _| theme.set(cx, Theme::light())),
        ))
        .p(24.0)
    }));
    app.click(role(Role::Button).name("Light"));
    app.advance_ms(16);
    assert!(app.is_animating());
    app.settle();
    assert!(app.frame_snapshot().contains("#1b1b2a12"));
}
