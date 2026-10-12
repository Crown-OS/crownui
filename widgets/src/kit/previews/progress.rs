use crate::kit::{
    preview::{Preview, stage},
    progress,
};

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "progress_empty",
            build: || stage(progress(0.0)),
        },
        Preview {
            name: "progress_partial",
            build: || stage(progress(0.42)),
        },
        Preview {
            name: "progress_full",
            build: || stage(progress(1.0)),
        },
    ]
}
