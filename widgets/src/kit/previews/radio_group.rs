use view::{View, component};

use crate::kit::{
    preview::{Preview, stage},
    radio_group,
};

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "radio_group_selected",
            build: || {
                component(|cx| {
                    let selected = cx.signal(Some(0));
                    stage(radio_group(["Default", "Comfortable", "Compact"], selected))
                })
                .boxed()
            },
        },
        Preview {
            name: "radio_group_empty",
            build: || {
                component(|cx| {
                    let selected = cx.signal(None);
                    stage(radio_group(["ext4", "btrfs", "xfs"], selected))
                })
                .boxed()
            },
        },
    ]
}
