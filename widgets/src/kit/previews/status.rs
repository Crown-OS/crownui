use crate::kit::{
    StatusOptions, Tone,
    preview::{Preview, stage},
    status,
};

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "status_success",
            build: || stage(status("Connected").tone(Tone::Success)),
        },
        Preview {
            name: "status_warning",
            build: || stage(status("Weak Security").tone(Tone::Warning)),
        },
        Preview {
            name: "status_danger",
            build: || stage(status("Couldn't join network").tone(Tone::Danger)),
        },
        Preview {
            name: "status_neutral",
            build: || stage(status("Not connected")),
        },
    ]
}
