use view::Styled;

use crate::{
    hstack,
    kit::{
        preview::{Preview, stage},
        spinner,
    },
    text,
};

pub(super) fn previews() -> Vec<Preview> {
    vec![Preview {
        name: "spinner",
        build: || {
            stage(
                hstack((spinner(), text("Looking for networks…")))
                    .gap(8.0)
                    .items_center(),
            )
        },
    }]
}
