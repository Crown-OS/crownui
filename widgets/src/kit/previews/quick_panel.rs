use view::{Styled, View, component};

use crate::{
    kit::{
        PanelHeaderOptions, PanelItemOptions, PanelSectionOptions, PanelSliderOptions,
        icon::tinted_icon,
        panel_action, panel_header, panel_item, panel_readout, panel_section, panel_separator,
        panel_slider,
        preview::{Preview, stage},
        quick_panel,
    },
    text::text,
};

const SPEAKER: &str =
    r#"<path d="M7 2h10a2 2 0 0 1 2 2v16a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2z"/>"#;

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "quick_panel_sound",
            build: || {
                component(|cx| {
                    let volume = cx.signal(0.6);
                    let enabled = cx.signal(true);
                    stage(quick_panel((
                        panel_header("Sound").switch(enabled),
                        panel_slider("Volume", volume)
                            .glyph(|tint| tinted_icon(SPEAKER, 2.0, tint).size(18.0).boxed()),
                        panel_separator(),
                        panel_section("Output"),
                        panel_item("MacBook Speakers")
                            .badge(|tint| tinted_icon(SPEAKER, 2.0, tint).size(16.0).boxed())
                            .selected(true),
                        panel_item("AirPods Pro")
                            .badge(|tint| tinted_icon(SPEAKER, 2.0, tint).size(16.0).boxed())
                            .detail("70%"),
                        panel_section("Other Devices").disclosure(),
                        panel_item("In use by Firefox").inert(),
                        panel_separator(),
                        panel_action("Sound Settings…"),
                    )))
                })
                .boxed()
            },
        },
        Preview {
            name: "quick_panel_readout",
            build: || {
                stage(quick_panel((
                    panel_readout("21:47", "Monday, 5 October"),
                    text("").h(0.0),
                )))
                .boxed()
            },
        },
    ]
}
