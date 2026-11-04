#![cfg(feature = "audio")]

use std::{
    sync::mpsc,
    time::{Duration, Instant},
};

use audio::{Enhancement, MicrophoneOptions, StreamState};
use view::{TaskExecutor, TaskFuture, TaskId, Ui, component};
use widgets::{kit::use_microphone, text};

struct ForwardingExecutor(mpsc::Sender<(TaskId, TaskFuture)>);

impl TaskExecutor for ForwardingExecutor {
    fn spawn(&mut self, id: TaskId, future: TaskFuture) {
        let _ = self.0.send((id, future));
    }

    fn cancel(&mut self, _id: TaskId) {}
}

#[test]
#[ignore = "opens the default microphone through PipeWire"]
fn use_microphone_mirrors_the_stream_into_signals() {
    let (found, signals) = mpsc::channel();
    let mut ui = Ui::new(component(move |cx| {
        let microphone = use_microphone(
            cx,
            MicrophoneOptions::default().enhancement(Enhancement::Off),
        );
        let _ = found.send(microphone);
        text("mic")
    }));
    let (forward, tasks) = mpsc::channel();
    ui.set_task_executor(Box::new(ForwardingExecutor(forward)));
    let microphone = signals.recv().expect("the component ran");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("a test runtime");

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut live = false;
    let mut heard = false;
    while Instant::now() < deadline && !(live && heard) {
        let Ok((id, future)) = tasks.recv_timeout(Duration::from_millis(100)) else {
            continue;
        };
        let waited = runtime
            .block_on(async { tokio::time::timeout(Duration::from_millis(200), future).await });
        if let Ok(output) = waited {
            ui.complete_task(id, output);
        }
        live = matches!(
            microphone.state.get(ui.cx()),
            StreamState::Live { enhanced_by: None }
        );
        heard |= microphone.level.get(ui.cx()).rms > 0.0;
    }
    assert!(live, "{:?}", microphone.state.get(ui.cx()));
    assert!(heard);
    assert!(microphone.take_frames(ui.cx()).is_some());
    assert!(microphone.take_frames(ui.cx()).is_none());
}
