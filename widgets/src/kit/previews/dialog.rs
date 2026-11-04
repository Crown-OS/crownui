use view::{Styled, View, component};

use crate::{
    kit::{
        ButtonOptions, ButtonVariant, button, dialog,
        preview::{Preview, stage},
    },
    spacer::spacer,
    stack::hstack,
    text::text,
};

pub(super) fn previews() -> Vec<Preview> {
    vec![Preview {
        name: "dialog_confirm",
        build: || {
            component(|cx| {
                let open = cx.signal(true);
                stage((
                    text("Documents"),
                    dialog(open, || {
                        (
                            text("Delete 3 items permanently?").font_semibold(),
                            text("This cannot be undone."),
                            hstack((
                                spacer(),
                                button("Cancel").variant(ButtonVariant::Secondary),
                                button("Delete").variant(ButtonVariant::Destructive),
                            ))
                            .gap(8.0),
                        )
                    }),
                ))
            })
            .boxed()
        },
    }]
}
