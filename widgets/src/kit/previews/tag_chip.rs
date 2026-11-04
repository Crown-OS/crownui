use geometry::rgb;
use view::Styled;

use crate::{
    kit::{
        preview::{Preview, stage},
        tag_chip,
    },
    stack::hstack,
};

pub(super) fn previews() -> Vec<Preview> {
    vec![Preview {
        name: "tag_chips",
        build: || {
            stage(
                hstack((
                    tag_chip("Red", rgb(0xe5484d)),
                    tag_chip("Work", rgb(0x3b82f6)),
                    tag_chip("Done", rgb(0x4ade80)),
                ))
                .gap(6.0),
            )
        },
    }]
}
