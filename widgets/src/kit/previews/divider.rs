use view::Styled;

use crate::{
    kit::{
        DividerOptions, divider,
        preview::{Preview, stage},
    },
    text::text,
};

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "divider_hairline",
            build: || stage((text("above"), divider().w(240.0), text("below"))),
        },
        Preview {
            name: "divider_spaced",
            build: || {
                stage((
                    text("above"),
                    divider().height(12.0).w(240.0),
                    text("below"),
                ))
            },
        },
    ]
}
