use view::{AnyView, Element, View, component};

use crate::kit::{
    SearchKind, SearchOptions, TextInputOptions,
    preview::{Preview, stage},
    search, suggestion,
};

fn search_stage(
    query: &'static str,
    configure: fn(Element<SearchKind>) -> Element<SearchKind>,
) -> AnyView {
    component(move |cx| {
        let query = cx.signal(query.to_owned());
        stage(configure(search(query)))
    })
    .boxed()
}

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "search_empty",
            build: || search_stage("", |bar| bar),
        },
        Preview {
            name: "search_with_query",
            build: || search_stage("blue", |bar| bar.width(320.0)),
        },
        Preview {
            name: "search_with_suggestions",
            build: || {
                component(|cx| {
                    let query = cx.signal(String::from("wi"));
                    let rows = cx.signal(vec![
                        suggestion("Wi-Fi").detail("Network"),
                        suggestion("Wi-Fi Calling").detail("Phone"),
                    ]);
                    stage(search(query).suggestions(rows))
                })
                .boxed()
            },
        },
    ]
}
