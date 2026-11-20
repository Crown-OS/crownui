use graphics::RecordingBackend;
use runtime::{AppConfig, RunError, Runner};
use taffy_layout::TaffyEngine;
use widgets::text;
use windowing::{Platform, WindowEvent, WindowId, headless::HeadlessPlatform};

type TestRunner = Runner<RecordingBackend, TaffyEngine>;

fn open(
    runner: &mut TestRunner,
    platform: &mut HeadlessPlatform,
    title: &str,
) -> Result<WindowId, RunError> {
    runner.open_window(
        platform,
        text(title),
        TaffyEngine::new(),
        AppConfig::new(title),
    )
}

fn started_surfaces() -> (TestRunner, HeadlessPlatform) {
    let mut runner = TestRunner::empty(true);
    let mut platform = HeadlessPlatform::default();
    platform.start(&mut runner);
    (runner, platform)
}

#[test]
fn closing_one_window_keeps_the_others_and_destroys_its_platform_window() -> Result<(), RunError> {
    let (mut runner, mut platform) = started_surfaces();
    let left = open(&mut runner, &mut platform, "Left")?;
    let right = open(&mut runner, &mut platform, "Right")?;
    platform.run_until_idle(&mut runner);

    platform.inject(left, WindowEvent::CloseRequested);
    platform.run_until_idle(&mut runner);

    assert!(!runner.has_window(left));
    assert!(platform.window(left).is_none());
    assert!(runner.has_window(right));
    assert!(!platform.exiting());
    Ok(())
}

#[test]
fn a_kept_alive_app_survives_its_last_window() -> Result<(), RunError> {
    let (mut runner, mut platform) = started_surfaces();
    let only = open(&mut runner, &mut platform, "Only")?;
    platform.run_until_idle(&mut runner);

    runner.close_window(&mut platform, only);
    platform.inject(only, WindowEvent::CloseRequested);
    platform.run_until_idle(&mut runner);

    assert!(platform.window(only).is_none());
    assert!(!platform.exiting());
    Ok(())
}

#[test]
fn an_empty_runner_without_keep_alive_exits_when_its_last_window_closes() -> Result<(), RunError> {
    let mut runner = TestRunner::empty(false);
    let mut platform = HeadlessPlatform::default();
    platform.start(&mut runner);
    let only = open(&mut runner, &mut platform, "Only")?;
    platform.inject(only, WindowEvent::CloseRequested);
    platform.run_until_idle(&mut runner);
    assert!(platform.exiting());
    Ok(())
}
