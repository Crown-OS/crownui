use view::Styled;

use crate::{
    kit::{
        card,
        preview::{Preview, stage},
        soft_shadow,
    },
    text::text,
};

pub(super) fn previews() -> Vec<Preview> {
    vec![Preview {
        name: "soft_shadow_card",
        build: || stage(soft_shadow(card(text("Network").m(12.0)), 14.0)),
    }]
}
