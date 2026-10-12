use reactive::Signal;
use view::{Cx, Semantic, component};

use crate::kit::{
    SelectOptions,
    preview::{Preview, stage},
    select,
};

const FRAMEWORKS: [&str; 5] = ["Next.js", "SvelteKit", "Nuxt.js", "Remix", "Astro"];

fn framework_signals(
    cx: &mut Cx,
    selected: Option<usize>,
) -> (Signal<Vec<String>>, Signal<Option<usize>>) {
    (
        cx.signal(FRAMEWORKS.map(String::from).to_vec()),
        cx.signal(selected),
    )
}

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "select_placeholder",
            build: || {
                stage(component(|cx| {
                    let (options, selected) = framework_signals(cx, None);
                    select(options, selected)
                        .placeholder("Select a framework")
                        .label("Framework")
                }))
            },
        },
        Preview {
            name: "select_value",
            build: || {
                stage(component(|cx| {
                    let (options, selected) = framework_signals(cx, Some(1));
                    select(options, selected).label("Framework")
                }))
            },
        },
        Preview {
            name: "select_disabled",
            build: || {
                stage(component(|cx| {
                    let (options, selected) = framework_signals(cx, Some(0));
                    select(options, selected).label("Framework").disabled(true)
                }))
            },
        },
        Preview {
            name: "select_open",
            build: || {
                stage(component(|cx| {
                    let (options, selected) = framework_signals(cx, Some(2));
                    select(options, selected).label("Framework").start_open()
                }))
            },
        },
    ]
}
