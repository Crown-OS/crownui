use view::{Prop, Styled};

use crate::{
    kit::{
        ButtonOptions, ButtonVariant, button,
        preview::{Preview, stage},
        wizard, wizard_footer, wizard_page, wizard_rail,
    },
    text,
};

const STEPS: [&str; 4] = ["Welcome", "Disk", "Review", "Restart"];
const WIDTH: f32 = 432.0;
const HEIGHT: f32 = 272.0;

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "wizard_rail",
            build: || stage(wizard_rail(STEPS, Prop::Static(2)).w(WIDTH)),
        },
        Preview {
            name: "wizard",
            build: || {
                stage(
                    wizard(
                        STEPS,
                        Prop::Static(1),
                        wizard_page(
                            "How much room should CrownOS get?",
                            Some("Windows keeps everything it is using now."),
                            text("Disk controls go here."),
                        ),
                        wizard_footer(
                            button("Back"),
                            button("Continue").variant(ButtonVariant::Primary),
                        ),
                    )
                    .w(WIDTH)
                    .h(HEIGHT),
                )
            },
        },
    ]
}
