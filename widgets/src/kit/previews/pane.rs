use view::Styled;

use crate::{
    kit::{
        PaneFill, PaneOptions, Radii, pane,
        preview::{Preview, stage},
    },
    stack::hstack,
    text::text,
};

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "pane_accent_split",
            build: || {
                stage(
                    hstack((
                        pane(text("Brand").p(24.0))
                            .accent_gradient()
                            .radii(Radii::left(24.0)),
                        pane(text("Details").p(24.0))
                            .themed_fill(|theme| PaneFill::Color(theme.surface.raised))
                            .radii(Radii::right(24.0)),
                    ))
                    .w(320.0),
                )
            },
        },
        Preview {
            name: "pane_bordered",
            build: || {
                stage(
                    pane(text("Sunken").p(16.0))
                        .themed_fill(|theme| PaneFill::Color(theme.surface.sunken))
                        .bordered_with(1.0, |theme| theme.surface.border)
                        .radii(Radii::all(12.0)),
                )
            },
        },
    ]
}
