use view::{Semantic, Styled, View, component};

use crate::{
    kit::{
        StatusOptions, Tone, divider,
        preview::{Preview, stage},
        section, setting_row, status,
    },
    toggle::toggle,
};

pub(super) fn previews() -> Vec<Preview> {
    vec![Preview {
        name: "section_connectivity",
        build: || {
            component(|cx| {
                let bluetooth = cx.signal(false);
                stage(
                    section(
                        "Connectivity",
                        (
                            setting_row("Bluetooth", toggle(bluetooth).label("Bluetooth")),
                            divider(),
                            setting_row("Network", status("Connected").tone(Tone::Success)),
                        ),
                    )
                    .w(360.0),
                )
            })
            .boxed()
        },
    }]
}
