use graphics::RecordingBackend;
use runtime::{AppConfig, Runner};
use taffy_layout::TaffyEngine;
use view::{View, component};
use widgets::text;
use windowing::{WindowEvent, WindowId, headless::HeadlessPlatform};

const FIRST_WINDOW: WindowId = WindowId::from_raw(1);

fn close_first_window(root: impl View) -> HeadlessPlatform {
    let mut runner = match Runner::<RecordingBackend, TaffyEngine>::new(
        root,
        TaffyEngine::new(),
        AppConfig::new("Close"),
    ) {
        Ok(runner) => runner,
        Err(error) => panic!("the recording backend cannot fail to start: {error}"),
    };
    let mut platform = HeadlessPlatform::default();
    platform.start(&mut runner);
    platform.inject(FIRST_WINDOW, WindowEvent::CloseRequested);
    platform.run_until_idle(&mut runner);
    platform
}

#[test]
fn the_close_button_ends_an_app_without_listeners() {
    assert!(close_first_window(text("Idle")).exiting());
}

#[test]
fn a_prevented_close_keeps_the_app_running() {
    let busy = component(|cx| {
        cx.on_close_requested(|_, event| event.prevent_default());
        text("Writing image…")
    });
    assert!(!close_first_window(busy).exiting());
}
