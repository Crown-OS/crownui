use view::{Semantic, Styled, View, component};

use crate::{
    kit::{
        preview::{Preview, stage},
        setting_row,
    },
    toggle::toggle,
};

pub(super) fn previews() -> Vec<Preview> {
    vec![Preview {
        name: "setting_row_toggle",
        build: || {
            component(|cx| {
                let enabled = cx.signal(true);
                stage(setting_row("Wi-Fi", toggle(enabled).label("Wi-Fi")).w(360.0))
            })
            .boxed()
        },
    }]
}
