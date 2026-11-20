#![cfg(all(feature = "tokio", target_os = "linux"))]

use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, ensure};
use graphics::RecordingBackend;
use runtime::{AppConfig, Runner};
use taffy_layout::TaffyEngine;
use view::{View, component};
use widgets::text;
use windowing::{WindowEvent, headless::HeadlessPlatform};

const TASK_THREAD_NAME: &str = "crownui-tasks";
const PATIENCE: Duration = Duration::from_secs(5);

fn live_task_threads() -> anyhow::Result<usize> {
    let mut count = 0;
    for task in std::fs::read_dir("/proc/self/task")? {
        let name = std::fs::read_to_string(task?.path().join("comm")).unwrap_or_default();
        count += usize::from(name.trim_end() == TASK_THREAD_NAME);
    }
    Ok(count)
}

fn wait_for_task_threads(expected: usize) -> anyhow::Result<usize> {
    let deadline = Instant::now() + PATIENCE;
    loop {
        let live = live_task_threads()?;
        if live == expected || Instant::now() >= deadline {
            return Ok(live);
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn wait_for_progress_past(progress: &AtomicUsize, seen: usize) -> bool {
    let deadline = Instant::now() + PATIENCE;
    while progress.load(Ordering::SeqCst) <= seen {
        if Instant::now() >= deadline {
            return false;
        }
        thread::sleep(Duration::from_millis(5));
    }
    true
}

struct DropSignal(mpsc::Sender<()>);

impl Drop for DropSignal {
    fn drop(&mut self) {
        let _ = self.0.send(());
    }
}

async fn keep_counting(progress: Arc<AtomicUsize>, _dropped: DropSignal) {
    loop {
        progress.fetch_add(1, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
}

fn counting_window(progress: Arc<AtomicUsize>, dropped: mpsc::Sender<()>) -> impl View {
    component(move |cx| {
        cx.spawn(keep_counting(progress, DropSignal(dropped)), |_, ()| {});
        text("Counting")
    })
}

#[derive(Clone, Copy)]
enum CloseBy {
    Runner,
    CloseRequest,
}

#[test]
fn closing_a_window_ends_its_task_thread_and_stops_its_futures() -> anyhow::Result<()> {
    let mut runner = Runner::<RecordingBackend, TaffyEngine>::empty(true);
    let mut platform = HeadlessPlatform::default();
    platform.start(&mut runner);
    let baseline = live_task_threads()?;

    for close_by in [CloseBy::Runner, CloseBy::CloseRequest].repeat(4) {
        let progress = Arc::new(AtomicUsize::new(0));
        let (dropped, future_dropped) = mpsc::channel();
        let window = runner.open_window(
            &mut platform,
            counting_window(Arc::clone(&progress), dropped),
            TaffyEngine::new(),
            AppConfig::new("Counting"),
        )?;
        platform.run_until_idle(&mut runner);
        ensure!(
            wait_for_progress_past(&progress, 0),
            "the window's future never ran"
        );
        ensure!(
            live_task_threads()? > baseline,
            "the open window has no {TASK_THREAD_NAME} thread"
        );

        match close_by {
            CloseBy::Runner => runner.close_window(&mut platform, window),
            CloseBy::CloseRequest => platform.inject(window, WindowEvent::CloseRequested),
        }
        platform.run_until_idle(&mut runner);
        ensure!(!runner.has_window(window), "the window stayed open");

        future_dropped
            .recv_timeout(PATIENCE)
            .context("the closed window's future was never dropped")?;
        let after_close = progress.load(Ordering::SeqCst);
        thread::sleep(Duration::from_millis(50));
        ensure!(
            progress.load(Ordering::SeqCst) == after_close,
            "the closed window's future still runs"
        );
    }

    let live = wait_for_task_threads(baseline)?;
    ensure!(
        live == baseline,
        "{live} task threads are alive after closing every window, {baseline} before"
    );
    Ok(())
}
