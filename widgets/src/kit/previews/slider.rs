use view::{View, component};

use crate::kit::{
    SliderOptions,
    preview::{Preview, stage},
    slider,
};

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "slider_half",
            build: || {
                component(|cx| {
                    let value = cx.signal(0.5);
                    stage(slider(value))
                })
                .boxed()
            },
        },
        Preview {
            name: "slider_stepped",
            build: || {
                component(|cx| {
                    let value = cx.signal(3.0);
                    stage(slider(value).range(0.0, 10.0).step(1.0))
                })
                .boxed()
            },
        },
    ]
}
