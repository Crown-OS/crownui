use view::{View, component};

use crate::kit::{
    Choice, choice_list, icons,
    preview::{Preview, stage},
};

pub(super) fn previews() -> Vec<Preview> {
    vec![Preview {
        name: "choice_list_disks",
        build: || {
            component(|cx| {
                let selected = cx.signal(Some(0));
                stage(choice_list(
                    [
                        Choice::new("/dev/nvme0n1", "476 GB · Windows 11 · 94 GiB free")
                            .icon(icons::HARD_DRIVE),
                        Choice::new("/dev/sdb", "no free space")
                            .icon(icons::HARD_DRIVE)
                            .disabled(),
                        Choice::new("ext4", ""),
                    ],
                    selected,
                ))
            })
            .boxed()
        },
    }]
}
