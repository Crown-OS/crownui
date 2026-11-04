use geometry::Point;
use view::{View, component};

use crate::{
    kit::{
        MenuRowOptions, context_menu_at, icons, menu_item, menu_separator,
        preview::{Preview, stage},
    },
    text::text,
};

pub(super) fn previews() -> Vec<Preview> {
    vec![Preview {
        name: "context_menu_at_flipped",
        build: || {
            component(|cx| {
                let position = cx.signal(Some(Point::new(400.0, 260.0)));
                stage((
                    text("Right-clicked near the bottom-right corner"),
                    context_menu_at(position, || {
                        (
                            menu_item("Open"),
                            menu_item("Copy").leading_icon(icons::COPY),
                            menu_separator(),
                            menu_item("Move to Trash"),
                        )
                    }),
                ))
            })
            .boxed()
        },
    }]
}
