use view::{Styled, View, component};

use crate::{
    kit::{
        ButtonOptions, ButtonVariant, SidebarItemOptions, icon_button, icons,
        preview::{Preview, stage},
        sidebar_item, use_theme,
    },
    stack::vstack,
    util::themed::themed,
};

const DOT_SIZE: f32 = 7.0;

pub(super) fn previews() -> Vec<Preview> {
    vec![Preview {
        name: "sidebar_item_trailing",
        build: || {
            component(|cx| {
                let theme = use_theme(cx);
                let online = vstack(())
                    .size(DOT_SIZE)
                    .rounded(DOT_SIZE / 2.0)
                    .bg(themed(theme, |theme| theme.status.success));
                stage(
                    vstack((
                        sidebar_item("Downloads", icons::DOWNLOAD, false).trailing_text("3"),
                        sidebar_item("USB Drive", icons::HARD_DRIVE, true).trailing(
                            icon_button(icons::ARROW_UP_FROM_LINE, "Eject")
                                .variant(ButtonVariant::Ghost),
                        ),
                        sidebar_item("Pixel 8", icons::SMARTPHONE, false).trailing(online),
                    ))
                    .w(240.0),
                )
            })
            .boxed()
        },
    }]
}
