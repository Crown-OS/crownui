use view::{Listeners, Styled, bind, component};

use crate::{
    kit::{
        icons,
        preview::{Preview, stage},
        sidebar, sidebar_brand, sidebar_column, sidebar_group, sidebar_item, sidebar_section,
        sidebar_separator, sidebar_subitem,
    },
    text,
};

const WIDTH: f32 = 432.0;
const HEIGHT: f32 = 272.0;

fn preview(group_open: bool) -> view::AnyView {
    stage(component(move |cx| {
        let page = cx.signal(1);
        let open = cx.signal(group_open);
        sidebar(
            (
                sidebar_brand("Acme", icons::HOUSE),
                sidebar_section(
                    "Personal",
                    (
                        sidebar_item(
                            "Inbox",
                            icons::INBOX,
                            bind(move |runtime| page.get(runtime) == 0),
                        )
                        .on_click(move |cx, _| page.set(cx, 0)),
                        sidebar_item(
                            "Tasks",
                            icons::CIRCLE_DOT,
                            bind(move |runtime| page.get(runtime) == 1),
                        )
                        .on_click(move |cx, _| page.set(cx, 1)),
                    ),
                ),
                (!group_open).then(sidebar_separator),
                sidebar_group(
                    "Finance",
                    icons::HOUSE,
                    open,
                    sidebar_column(sidebar_subitem("Local currency", false)),
                ),
            ),
            text("Tasks"),
        )
        .w(WIDTH)
        .h(HEIGHT)
    }))
}

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "sidebar",
            build: || preview(true),
        },
        Preview {
            name: "sidebar_group_collapsed",
            build: || preview(false),
        },
    ]
}
