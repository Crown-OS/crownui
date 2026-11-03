use crowntest::TestApp;
use view::{View, component};
use widgets::{
    kit::{Theme, WindowRadius, provide_theme, provide_window_radius, sidebar},
    text,
};

fn shell_in_window(radius: f32) -> impl View {
    component(move |cx| {
        provide_theme(cx, Theme::dark());
        provide_window_radius(cx, WindowRadius(radius));
        sidebar(text("Menu"), text("Body"))
    })
}

#[test]
fn the_card_rounds_concentrically_with_the_window() {
    let frame = TestApp::new(shell_in_window(20.0)).frame_snapshot();
    assert!(frame.contains("fill 0,0 516x576 r8 #0f0f10cc"), "{frame}");
}

#[test]
fn a_window_tighter_than_the_inset_leaves_the_card_square() {
    let frame = TestApp::new(shell_in_window(8.0)).frame_snapshot();
    assert!(frame.contains("fill 0,0 516x576 #0f0f10cc"), "{frame}");
}
