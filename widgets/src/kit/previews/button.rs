use crate::kit::{
    ButtonOptions, ButtonSize, ButtonVariant, button, icon_button, icons,
    preview::{Preview, stage},
};

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "button_primary",
            build: || stage(button("Save").variant(ButtonVariant::Primary)),
        },
        Preview {
            name: "button_secondary",
            build: || stage(button("Details…")),
        },
        Preview {
            name: "button_ghost",
            build: || stage(button("More").variant(ButtonVariant::Ghost)),
        },
        Preview {
            name: "button_destructive",
            build: || stage(button("Forget This Network…").variant(ButtonVariant::Destructive)),
        },
        Preview {
            name: "button_tinted",
            build: || stage(button("Allow").variant(ButtonVariant::Tinted)),
        },
        Preview {
            name: "button_small_with_icon",
            build: || {
                stage(
                    button("Add")
                        .variant(ButtonVariant::Primary)
                        .control_size(ButtonSize::Small)
                        .leading_icon(icons::PLUS),
                )
            },
        },
        Preview {
            name: "button_icon",
            build: || stage(icon_button(icons::SETTINGS, "Settings").variant(ButtonVariant::Ghost)),
        },
    ]
}
