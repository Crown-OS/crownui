use view::{Styled, View, component};

use crate::kit::{
    PathSegment, icons, path_bar,
    preview::{Preview, stage},
};

fn home_path() -> Vec<PathSegment> {
    vec![
        PathSegment::new("Home").icon(icons::HOUSE),
        PathSegment::new("Documents"),
        PathSegment::new("Projects"),
        PathSegment::new("crownos"),
    ]
}

fn preview(width: f32) -> view::AnyView {
    component(move |cx| {
        let segments = cx.signal(home_path());
        stage(crate::stack::hstack(path_bar(segments, |_, _| {})).w(width))
    })
    .boxed()
}

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "path_bar",
            build: || preview(420.0),
        },
        Preview {
            name: "path_bar_folded",
            build: || preview(200.0),
        },
    ]
}
