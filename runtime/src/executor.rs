use std::{any::Any, collections::HashMap};

use crossbeam_channel::Receiver;
use view::{TaskExecutor, TaskFuture, TaskId};
use windowing::Waker;

enum TaskCommand {
    Spawn(TaskId, TaskFuture),
    Cancel(TaskId),
    Finished(TaskId),
}

/// A finished task's output, waiting to be delivered on the UI thread.
pub(crate) type TaskOutput = (TaskId, Box<dyn Any + Send>);

/// Runs `cx.spawn` futures on a current-thread tokio runtime in its own thread.
pub(crate) struct TokioExecutor {
    commands: tokio::sync::mpsc::UnboundedSender<TaskCommand>,
}

impl TokioExecutor {
    /// Starts the runtime thread; finished outputs arrive on the returned receiver and `wake`
    /// is called after each one.
    pub(crate) fn start(wake: Waker) -> std::io::Result<(Self, Receiver<TaskOutput>)> {
        let (commands, mut incoming) = tokio::sync::mpsc::unbounded_channel::<TaskCommand>();
        let (outputs, finished) = crossbeam_channel::unbounded::<TaskOutput>();
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
        let loopback = commands.clone();
        std::thread::Builder::new().name("crownui-tasks".into()).spawn(move || {
            runtime.block_on(async move {
                let mut running: HashMap<TaskId, tokio::task::AbortHandle> = HashMap::new();
                while let Some(command) = incoming.recv().await {
                    match command {
                        TaskCommand::Spawn(id, future) => {
                            let outputs = outputs.clone();
                            let loopback = loopback.clone();
                            let handle = tokio::spawn(async move {
                                let output = future.await;
                                let _ = outputs.send((id, output));
                                let _ = loopback.send(TaskCommand::Finished(id));
                            });
                            running.insert(id, handle.abort_handle());
                        }
                        TaskCommand::Cancel(id) => {
                            if let Some(handle) = running.remove(&id) {
                                handle.abort();
                            }
                        }
                        TaskCommand::Finished(id) => {
                            running.remove(&id);
                            wake();
                        }
                    }
                }
            });
        })?;
        Ok((Self { commands }, finished))
    }
}

impl TaskExecutor for TokioExecutor {
    fn spawn(&mut self, id: TaskId, future: TaskFuture) {
        let _ = self.commands.send(TaskCommand::Spawn(id, future));
    }

    fn cancel(&mut self, id: TaskId) {
        let _ = self.commands.send(TaskCommand::Cancel(id));
    }
}
