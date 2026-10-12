use view::Styled;

use crate::kit::{
    CalloutOptions, Tone, callout,
    preview::{Preview, stage},
};

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "callout_warning_with_body",
            build: || {
                stage(
                    callout(Tone::Warning, "BitLocker is on")
                        .body(
                            "Suspend BitLocker before resizing this volume, or Windows will ask for your recovery key on the next boot.",
                        )
                        .max_w(360.0),
                )
            },
        },
        Preview {
            name: "callout_danger_title_only",
            build: || stage(callout(Tone::Danger, "Couldn't join network")),
        },
        Preview {
            name: "callout_success_with_body",
            build: || {
                stage(callout(Tone::Success, "Backup finished").body("Everything was copied."))
            },
        },
        Preview {
            name: "callout_neutral",
            build: || stage(callout(Tone::Neutral, "Heads up").body("Nothing to do.")),
        },
    ]
}
