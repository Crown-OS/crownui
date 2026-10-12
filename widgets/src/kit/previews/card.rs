use view::Styled;

use crate::{
    kit::{
        StatusOptions, Tone, card, divider,
        preview::{Preview, stage},
        status,
    },
    text::text,
};

pub(super) fn previews() -> Vec<Preview> {
    vec![Preview {
        name: "card_with_rows",
        build: || {
            stage(
                card((
                    text("Network").p(12.0),
                    divider(),
                    status("Connected").tone(Tone::Success).m(12.0),
                ))
                .w(280.0),
            )
        },
    }]
}
