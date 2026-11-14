use crowntest::TestApp;
use view::Styled;
use widgets::kit::{sidebar_with_header, title_bar_spacer};
use widgets::text;

fn window_fill(app: &TestApp) -> String {
    app.frame_snapshot()
        .lines()
        .find(|line| line.trim_start().starts_with("fill 0,0"))
        .map(str::trim)
        .unwrap_or_default()
        .to_owned()
}

/// A tiling compositor resizes a window right after it maps; the content has to fill the new
/// size in that frame, not spring into it over the clear color.
#[test]
fn a_resized_window_fills_its_new_size_at_once() {
    let mut app = TestApp::with_size(
        sidebar_with_header(
            (text("Menu"),),
            (title_bar_spacer(),),
            (text("Body").flex_1(),),
        ),
        1040.0,
        720.0,
    );
    app.settle();
    assert!(window_fill(&app).starts_with("fill 0,0 1040x720"));

    app.resize(1500, 900);
    app.advance_ms(16);
    assert!(
        window_fill(&app).starts_with("fill 0,0 1500x900"),
        "{}",
        window_fill(&app)
    );
}
