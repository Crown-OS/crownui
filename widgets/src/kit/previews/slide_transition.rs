use view::{Styled, component};

use crate::{
    kit::{
        preview::{Preview, stage},
        slide_transition,
    },
    text, vstack,
};

pub(super) fn previews() -> Vec<Preview> {
    vec![Preview {
        name: "slide_transition",
        build: || {
            stage(component(|cx| {
                let page = cx.signal(0);
                slide_transition(page, |_, ordinal| {
                    vstack(text(format!("Page {}", ordinal + 1)))
                        .w(320.0)
                        .h(96.0)
                })
            }))
        },
    }]
}
