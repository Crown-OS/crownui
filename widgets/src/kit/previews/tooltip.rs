use crate::kit::{
    button, icon_button, icons,
    preview::{Preview, stage},
    tooltip,
    tooltip::shown_tooltip,
};

pub(super) fn previews() -> Vec<Preview> {
    vec![Preview {
        name: "tooltip_shown",
        build: || {
            stage((
                shown_tooltip("Copy to the clipboard", icon_button(icons::COPY, "Copy")),
                tooltip("Hidden until hovered", button("Hover me")),
            ))
        },
    }]
}
