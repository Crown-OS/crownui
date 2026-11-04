use std::sync::mpsc;

use crowntest::TestApp;
use reactive::Signal;
use view::{Styled, TaskExecutor, TaskFuture, TaskId, View, bind, component, for_each, show};
use widgets::kit::{icons, sidebar, sidebar_item, sidebar_section};
use widgets::{spacer, text as label, vstack};

/// Starts nothing: the test completes each task by hand, as a backend worker finishing would.
struct HeldTasks(mpsc::Sender<TaskId>);

impl TaskExecutor for HeldTasks {
    fn spawn(&mut self, id: TaskId, _future: TaskFuture) {
        let _ = self.0.send(id);
    }

    fn cancel(&mut self, _id: TaskId) {}
}

fn disk_section(disks: Signal<Vec<String>>, title: &'static str) -> impl View {
    show(
        bind(move |runtime| disks.with(runtime, |disks| !disks.is_empty())),
        move || {
            sidebar_section(
                title,
                for_each(disks, String::clone, |cx, disk| {
                    let name = disk.with_untracked(cx, String::clone);
                    sidebar_item(name, icons::LAYOUT_GRID, false)
                }),
            )
        },
        || vstack(()),
    )
    .flex_col()
}

/// The disk manager's sidebar: sections mount inside the transitioned menu column when a task
/// reports the disks, pushing the rows below them down.
fn disks_sidebar(loaded_from_start: bool) -> impl View {
    component(move |cx| {
        let loaded = || {
            (
                vec!["Disk A".to_owned(), "Disk B".to_owned()],
                vec!["USB".to_owned()],
            )
        };
        let (internal, external) = if loaded_from_start {
            loaded()
        } else {
            (Vec::new(), Vec::new())
        };
        let internal = cx.signal(internal);
        let external = cx.signal(external);
        cx.spawn(async {}, move |cx, ()| {
            let (internal_disks, external_disks) = loaded();
            internal.set(cx, internal_disks);
            external.set(cx, external_disks);
        });
        sidebar(
            (
                sidebar_item("Overview", icons::LAYOUT_GRID, true),
                vec![
                    disk_section(internal, "Internal").boxed(),
                    disk_section(external, "External").boxed(),
                ],
                spacer(),
                sidebar_item("Open Image", icons::PLUS, false),
            ),
            label("content"),
        )
    })
}

fn settled_app(loaded_from_start: bool) -> (TestApp, Option<TaskId>) {
    let mut app = TestApp::with_size(disks_sidebar(loaded_from_start), 900.0, 600.0);
    let (sender, started) = mpsc::channel();
    app.ui().set_task_executor(Box::new(HeldTasks(sender)));
    app.settle();
    (app, started.try_recv().ok())
}

#[test]
fn rows_a_task_inserts_into_a_transitioned_column_settle_where_layout_puts_them() {
    let (expected, _) = settled_app(true);
    let (mut app, task) = settled_app(false);
    let Some(task) = task else {
        panic!("the sidebar spawned its task");
    };
    app.ui().complete_task(task, Box::new(()));
    app.advance_ms(16);
    assert!(
        app.is_animating(),
        "rows below the new sections move to make room"
    );
    assert!(app.frame_snapshot().contains("text \"USB\""));
    app.settle();
    assert!(!app.is_animating());
    assert_eq!(app.frame_snapshot(), expected.frame_snapshot());
}
