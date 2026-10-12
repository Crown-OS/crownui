use std::{any::Any, collections::HashMap, future::Future, pin::Pin};

use reactive::ScopeId;

use crate::Cx;

/// Identifies a spawned task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TaskId(pub u64);

/// A type-erased task future whose output is handed back to the UI thread.
pub type TaskFuture = Pin<Box<dyn Future<Output = Box<dyn Any + Send>> + Send>>;

/// Runs futures off the UI thread; the runtime provides one (tokio by default).
pub trait TaskExecutor {
    /// Starts `future`; its output must later reach [`crate::Ui::complete_task`].
    fn spawn(&mut self, id: TaskId, future: TaskFuture);
    /// Cancels a task whose component unmounted.
    fn cancel(&mut self, id: TaskId);
}

type Completion = Box<dyn FnOnce(&mut Cx, Box<dyn Any + Send>)>;

struct PendingTask {
    scope: ScopeId,
    on_done: Completion,
}

#[derive(Default)]
pub(crate) struct Tasks {
    executor: Option<Box<dyn TaskExecutor>>,
    next_id: u64,
    pending: HashMap<TaskId, PendingTask>,
    /// Futures spawned before the runtime installed an executor (the first build runs before the
    /// window exists), started as soon as one arrives.
    unstarted: Vec<(TaskId, TaskFuture)>,
}

impl Tasks {
    pub(crate) fn set_executor(&mut self, mut executor: Box<dyn TaskExecutor>) {
        for (id, future) in self.unstarted.drain(..) {
            executor.spawn(id, future);
        }
        self.executor = Some(executor);
    }

    pub(crate) fn cancel_dead(&mut self, is_alive: impl Fn(ScopeId) -> bool) {
        let dead: Vec<TaskId> = self
            .pending
            .iter()
            .filter(|(_, task)| !is_alive(task.scope))
            .map(|(id, _)| *id)
            .collect();
        for id in dead {
            self.pending.remove(&id);
            self.unstarted.retain(|(unstarted, _)| *unstarted != id);
            if let Some(executor) = self.executor.as_mut() {
                executor.cancel(id);
            }
        }
    }

    pub(crate) fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

impl Cx {
    /// Runs `future` off the UI thread and calls `on_done` with its output back on the UI thread,
    /// unless the calling component unmounted first (then the task is cancelled). A task spawned
    /// before the runtime installed its executor starts when the executor arrives.
    pub fn spawn<T: Send + 'static>(
        &mut self,
        future: impl Future<Output = T> + Send + 'static,
        on_done: impl FnOnce(&mut Cx, T) + 'static,
    ) -> TaskId {
        let id = TaskId(self.tasks.next_id);
        self.tasks.next_id += 1;
        let scope = self.current_scope();
        let future: TaskFuture =
            Box::pin(async move { Box::new(future.await) as Box<dyn Any + Send> });
        match self.tasks.executor.as_mut() {
            Some(executor) => executor.spawn(id, future),
            None => self.tasks.unstarted.push((id, future)),
        }
        self.tasks.pending.insert(
            id,
            PendingTask {
                scope,
                on_done: Box::new(move |cx, output| {
                    if let Ok(output) = output.downcast::<T>() {
                        on_done(cx, *output);
                    }
                }),
            },
        );
        id
    }

    pub(crate) fn complete_task(&mut self, id: TaskId, output: Box<dyn Any + Send>) {
        let Some(task) = self.tasks.pending.remove(&id) else {
            return;
        };
        if self.is_scope_alive(task.scope) {
            let previous = self.set_current_scope(task.scope);
            (task.on_done)(self, output);
            self.set_current_scope(previous);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;
    use crate::{Element, Kind, NodeId, Ui, component};

    struct Recorder(mpsc::Sender<TaskId>);

    impl TaskExecutor for Recorder {
        fn spawn(&mut self, id: TaskId, _future: TaskFuture) {
            let _ = self.0.send(id);
        }

        fn cancel(&mut self, _id: TaskId) {}
    }

    struct Blank;

    impl Kind for Blank {
        const NAME: &'static str = "blank";

        fn mount(self, _cx: &mut Cx, _node: NodeId) {}
    }

    #[test]
    fn tasks_spawned_during_the_first_build_start_once_an_executor_arrives() {
        let mut ui = Ui::new(component(|cx| {
            cx.spawn(async {}, |_, ()| {});
            Element::new(Blank)
        }));
        let (sender, started) = mpsc::channel();
        ui.set_task_executor(Box::new(Recorder(sender)));
        assert_eq!(started.try_iter().count(), 1);
    }
}
