use view::{View, component};

use crate::{
    kit::{
        BadgeOptions, badge, icons,
        preview::{Preview, stage},
        use_theme,
    },
    util::themed::themed,
};

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "badge_warning",
            build: || {
                component(|cx| {
                    let theme = use_theme(cx);
                    stage(badge(
                        "Weak Security",
                        themed(theme, |theme| theme.status.warning),
                    ))
                })
                .boxed()
            },
        },
        Preview {
            name: "badge_accent_with_icon",
            build: || {
                component(|cx| {
                    let theme = use_theme(cx);
                    stage(
                        badge("Bluetooth", themed(theme, |theme| theme.accent.end))
                            .leading_icon(icons::BLUETOOTH),
                    )
                })
                .boxed()
            },
        },
    ]
}
