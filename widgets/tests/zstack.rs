use crowntest::TestApp;
use view::Styled;
use widgets::{vstack, zstack};

#[test]
fn an_overlay_fills_the_stack_unless_its_author_placed_it() {
    let mut app = TestApp::with_size(
        zstack((
            vstack(()).h(40.0),
            vstack(()),
            vstack(())
                .absolute()
                .top(28.0)
                .left(0.0)
                .right(0.0)
                .bottom(0.0),
        )),
        400.0,
        300.0,
    );
    let layout = app.layout_snapshot();
    assert!(layout.contains("stack > stack[1] @0,0 400x300"), "{layout}");
    assert!(
        layout.contains("stack > stack[2] @0,28 400x272"),
        "{layout}"
    );
}
