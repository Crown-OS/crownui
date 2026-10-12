use view::Semantic;

use crate::kit::{
    PopupMenuOptions, icons, popup_menu, popup_menu_item, popup_menu_separator,
    preview::{Preview, stage},
};

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "popup_menu_closed",
            build: || {
                stage(popup_menu(
                    icons::ELLIPSIS,
                    "Network options",
                    vec![popup_menu_item("Open", |_| {})],
                ))
            },
        },
        Preview {
            name: "popup_menu_disabled",
            build: || {
                stage(
                    popup_menu(
                        icons::ELLIPSIS,
                        "Network options",
                        vec![popup_menu_item("Open", |_| {})],
                    )
                    .disabled(true),
                )
            },
        },
        Preview {
            name: "popup_menu_open",
            build: || {
                stage(
                    popup_menu(
                        icons::ELLIPSIS,
                        "Network options",
                        vec![
                            popup_menu_item("Auto-Join", |_| {}).checked(true),
                            popup_menu_item("Network Settings…", |_| {}),
                            popup_menu_separator(),
                            popup_menu_item("Share Password…", |_| {}).disabled(true),
                            popup_menu_item("Forget This Network…", |_| {}).destructive(),
                        ],
                    )
                    .start_open(),
                )
            },
        },
    ]
}
