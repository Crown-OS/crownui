use crate::kit::{
    MenuRowOptions, context_menu, icons, menu_item, menu_item_disabled, menu_item_selected,
    menu_separator,
    preview::{Preview, stage},
    preview_menu,
};

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "context_menu_reference",
            build: || stage(preview_menu()),
        },
        Preview {
            name: "context_menu_compact",
            build: || {
                stage(context_menu((
                    menu_item("Copy")
                        .leading_icon(icons::COPY)
                        .shortcut_hint("⌘ C"),
                    menu_item_selected("Cut")
                        .leading_icon(icons::SCISSORS)
                        .shortcut_hint("⌘ X"),
                    menu_separator(),
                    menu_item_disabled("Paste").leading_icon(icons::CLIPBOARD),
                )))
            },
        },
    ]
}
