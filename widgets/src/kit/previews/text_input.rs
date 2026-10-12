use view::{AnyView, Element, Semantic, View, component};

use crate::kit::{
    ClearButton, FieldShape, TextInputKind, TextInputOptions, icons,
    preview::{Preview, stage},
    text_input,
};

fn field_stage(
    initial: &'static str,
    configure: fn(Element<TextInputKind>) -> Element<TextInputKind>,
) -> AnyView {
    component(move |cx| {
        let value = cx.signal(initial.to_owned());
        stage(configure(text_input(value)))
    })
    .boxed()
}

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "text_input_placeholder",
            build: || field_stage("", |field| field.placeholder("Network name")),
        },
        Preview {
            name: "text_input_filled",
            build: || field_stage("Studio Wi-Fi", |field| field),
        },
        Preview {
            name: "text_input_secret",
            build: || field_stage("hunter2", |field| field.placeholder("Password").secret()),
        },
        Preview {
            name: "text_input_search_pill",
            build: || {
                field_stage("wifi", |field| {
                    field
                        .shape(FieldShape::Pill)
                        .leading_icon(icons::SEARCH)
                        .clear_button(ClearButton::Visible)
                })
            },
        },
        Preview {
            name: "text_input_read_only",
            build: || field_stage("Fixed value", |field| field.read_only()),
        },
        Preview {
            name: "text_input_disabled",
            build: || field_stage("Unavailable", |field| field.disabled(true)),
        },
    ]
}
