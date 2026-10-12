use crownui::prelude::*;

#[derive(Clone, PartialEq)]
struct Todo {
    id: u32,
    title: String,
    done: bool,
}

fn todo_row(todos: Signal<Vec<Todo>>, todo: Signal<Todo>) -> impl View {
    hstack((
        toggle_done(todos, todo),
        text(bind(move |rt| todo.with(rt, |todo| todo.title.clone())))
            .grow(1.0)
            .text_color(bind(move |rt| {
                if todo.with(rt, |todo| todo.done) { rgb(0x94a3b8) } else { rgb(0x0f172a) }
            })),
        button("✕")
            .label(bind(move |rt| todo.with(rt, |todo| format!("Delete {}", todo.title))))
            .bg(TRANSPARENT)
            .hover(|style| style.bg(rgba(0xef444420)))
            .on_click(move |cx, _| {
                let id = todo.with(cx, |todo| todo.id);
                todos.update(cx, |list| list.retain(|item| item.id != id));
            }),
    ))
    .gap(10.0)
    .items_center()
    .px(12.0)
    .py(8.0)
    .rounded(10.0)
    .bg(WHITE)
    .shadow(Shadow::sm())
    .enter(|style| style.opacity(0.0).translate_y(-8.0))
    .transition(Spring::smooth())
}

fn toggle_done(todos: Signal<Vec<Todo>>, todo: Signal<Todo>) -> impl View {
    component(move |cx| {
        let initially_done = todo.with(cx, |todo| todo.done);
        let done = cx.signal(initially_done);
        cx.effect(move |rt| {
            let checked = done.get(rt);
            let id = todo.with_untracked(rt, |todo| todo.id);
            todos.update(rt, |list| {
                if let Some(item) = list.iter_mut().find(|item| item.id == id) {
                    item.done = checked;
                }
            });
        });
        toggle(done).label(bind(move |rt| todo.with(rt, |todo| format!("Done: {}", todo.title))))
    })
}

fn main() -> Result<(), crownui::RunError> {
    tracing_subscriber::fmt::init();
    crownui::run(
        component(|cx| {
            let todos = cx.signal(Vec::<Todo>::new());
            let draft = cx.signal(String::new());
            let next_id = cx.signal(1_u32);
            let add = move |cx: &mut Cx| {
                let title = draft.get(cx);
                if title.trim().is_empty() {
                    return;
                }
                let id = next_id.get(cx);
                next_id.set(cx, id + 1);
                todos.update(cx, |list| list.push(Todo { id, title, done: false }));
                draft.set(cx, String::new());
            };
            vstack((
                text("Todos").text_size(28.0).font_bold(),
                hstack((
                    text_field(draft, "What needs doing?")
                        .grow(1.0)
                        .label("New todo")
                        .on_submit(move |cx, _| add(cx)),
                    button("Add")
                        .bg(rgb(0x3b82f6))
                        .text_color(WHITE)
                        .on_click(move |cx, _| add(cx)),
                ))
                .gap(8.0),
                scroll_view(for_each(todos, |todo: &Todo| todo.id, move |_cx, todo| todo_row(todos, todo)).gap(8.0))
                    .grow(1.0)
                    .label("Todo list"),
                text(bind(move |rt| {
                    let left = todos.with(rt, |list| list.iter().filter(|todo| !todo.done).count());
                    format!("{left} left")
                }))
                .text_color(rgb(0x64748b)),
            ))
            .p(24.0)
            .gap(16.0)
            .bg(rgb(0xf1f5f9))
        }),
        AppConfig::new("Todos").size(480, 640).mcp(true),
    )
}
