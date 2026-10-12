use view::{View, component};

use crate::kit::{
    ToggleOptions,
    preview::{Preview, stage},
    toggle,
};

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "toggle_off",
            build: || {
                component(|cx| {
                    let on = cx.signal(false);
                    stage(toggle(on).caption("Bluetooth"))
                })
                .boxed()
            },
        },
        Preview {
            name: "toggle_on",
            build: || {
                component(|cx| {
                    let on = cx.signal(true);
                    stage(toggle(on).caption("Wi-Fi"))
                })
                .boxed()
            },
        },
    ]
}
