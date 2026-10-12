use crowntest::{Role, TestApp, role, text as text_query};
use geometry::rgb;
use view::{Listeners, Styled, component};
use widgets::{
    kit::{GradientStops, PaneOptions, Radii, Theme, button, pane, provide_theme},
    text, vstack,
};

fn stage(theme: Theme, content: impl view::ViewSeq) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, theme);
        vstack(content).p(24.0).items_start()
    }))
}

#[test]
fn an_unstyled_pane_paints_nothing_and_hugs_its_content() {
    let mut app = stage(Theme::dark(), pane(text("Brand")));
    let label = app.find(text_query("Brand")).bounds;
    assert_eq!((label.origin.x, label.origin.y), (24.0, 24.0));
    let frame = app.frame_snapshot();
    assert!(
        !frame.contains("fill") && !frame.contains("stroke"),
        "{frame}"
    );
}

#[test]
fn a_flat_fill_follows_the_per_corner_radii() {
    let mut app = stage(
        Theme::dark(),
        pane(text("Brand"))
            .fill(rgb(0xff0000))
            .radii(Radii::left(24.0)),
    );
    insta::assert_snapshot!(app.snapshot());
}

#[test]
fn a_gradient_fill_runs_top_to_bottom_over_the_pane() {
    let stops = GradientStops::new(rgb(0x0000ff), rgb(0x00ff00));
    let app = stage(Theme::dark(), pane(text("Brand")).gradient(stops));
    let frame = app.frame_snapshot();
    assert!(
        frame.contains("gradient(#0000ffff@0,#00ff00ff@1)"),
        "{frame}"
    );
}

#[test]
fn the_accent_gradient_follows_the_theme() {
    let app = stage(Theme::dark(), pane(text("Brand")).accent_gradient());
    assert!(
        app.frame_snapshot()
            .contains("gradient(#8f6dfbff@0,#6d48e8ff@1)")
    );
}

#[test]
fn the_border_is_inset_by_half_its_width_and_shrinks_the_radii() {
    let app = stage(
        Theme::dark(),
        pane(text("Brand"))
            .fill(rgb(0x222222))
            .radii(Radii::all(10.0))
            .bordered(rgb(0xffffff), 2.0),
    );
    let frame = app.frame_snapshot();
    assert!(frame.contains("stroke2 1,1"), "{frame}");
    assert!(frame.contains("r9"), "{frame}");
}

#[test]
fn a_themed_border_follows_the_theme() {
    let mut app = TestApp::new(component(|cx| {
        let theme = provide_theme(cx, Theme::light());
        vstack((
            pane(text("Brand")).bordered_with(1.0, |theme| theme.surface.border),
            button("Dark").on_click(move |cx, _| theme.set(cx, Theme::dark())),
        ))
        .p(24.0)
    }));
    assert!(app.frame_snapshot().contains("#1b1b2a12"));
    app.click(role(Role::Button).name("Dark"));
    app.settle();
    assert!(app.frame_snapshot().contains("#27272aff"));
}

#[test]
fn content_is_clipped_to_the_pane() {
    let mut app = stage(
        Theme::dark(),
        pane(text("Brand"))
            .fill(rgb(0xff0000))
            .radii(Radii::all(8.0)),
    );
    let layout = app.layout_snapshot();
    assert!(layout.contains("kit_pane"), "{layout}");
    assert!(app.frame_snapshot().contains("clip"));
}
