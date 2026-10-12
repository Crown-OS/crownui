use view::Styled;

use crate::{
    kit::{
        ButtonOptions, ButtonVariant, HeroStep, button, card, divider, hero_pane, hero_wizard,
        icon_colored, icons,
        preview::{Preview, stage},
    },
    text::text,
};

const PAGE_WIDTH: f32 = 934.0;
const PAGE_HEIGHT: f32 = 622.0;
const BRAND_SIZE: f32 = 96.0;
const BRAND_STROKE: f32 = 1.5;

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "hero_wizard_disk_step",
            build: || {
                stage(
                    hero_wizard(
                        hero_pane(
                            icon_colored(icons::CROWN, BRAND_STROKE, |theme| theme.text.on_accent)
                                .size(BRAND_SIZE),
                        ),
                        HeroStep::new("Please select the disk to install to")
                            .subtitle(
                                "After selecting the disk, click \u{201c}Next\u{201d} and continue with the installation",
                            )
                            .steps(4, 1_usize),
                        card((
                            text("Samsung SSD 990 PRO 1 TB").p(14.0),
                            divider(),
                            text("WD Blue SN580 500 GB").p(14.0),
                        )),
                        button("Next step").variant(ButtonVariant::Primary).stretch(),
                    )
                    .w(PAGE_WIDTH)
                    .h(PAGE_HEIGHT),
                )
            },
        },
        Preview {
            name: "hero_wizard_terminal_page",
            build: || {
                stage(
                    hero_wizard(
                        hero_pane(
                            icon_colored(icons::CROWN, BRAND_STROKE, |theme| theme.text.on_accent)
                                .size(BRAND_SIZE),
                        ),
                        HeroStep::new("Installation complete"),
                        (),
                        button("Restart now")
                            .variant(ButtonVariant::Primary)
                            .stretch(),
                    )
                    .w(PAGE_WIDTH)
                    .h(PAGE_HEIGHT),
                )
            },
        },
    ]
}
