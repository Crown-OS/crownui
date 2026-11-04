use crate::kit::{
    ToastOptions, icons,
    preview::{Preview, stage},
    toast,
};

pub(super) fn previews() -> Vec<Preview> {
    vec![Preview {
        name: "toast_messages",
        build: || {
            stage((
                toast("Copied 3 items").leading_icon(icons::COPY),
                toast("Mounted USB Drive").leading_icon(icons::HARD_DRIVE),
                toast("Moved to Trash"),
            ))
        },
    }]
}
