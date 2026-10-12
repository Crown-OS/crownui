use view::{View, component};

use crate::kit::{
    ToggleGroupOptions, ToggleItem, icons,
    preview::{Preview, stage},
    toggle_group,
};

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "toggle_group_text",
            build: || {
                component(|cx| {
                    let selected = cx.signal(1);
                    stage(toggle_group(
                        [
                            ToggleItem::text("Default"),
                            ToggleItem::text("Dark"),
                            ToggleItem::text("Mono"),
                        ],
                        selected,
                    ))
                })
                .boxed()
            },
        },
        Preview {
            name: "toggle_group_icons_stretched",
            build: || {
                component(|cx| {
                    let selected = cx.signal(0);
                    stage(
                        toggle_group(
                            [
                                ToggleItem::icon(icons::SUN, "Light"),
                                ToggleItem::icon(icons::MOON, "Dark"),
                            ],
                            selected,
                        )
                        .stretch(),
                    )
                })
                .boxed()
            },
        },
    ]
}
