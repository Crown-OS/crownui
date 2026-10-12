use geometry::Color;
use view::{Styled, View, component};

use crate::kit::{
    Segment, SegmentedBarOptions,
    preview::{Preview, stage},
    segmented_bar, use_theme,
};

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "segmented_bar_disk_usage",
            build: || {
                component(|cx| {
                    let theme = use_theme(cx);
                    stage(
                        segmented_bar([])
                            .segments_from(move |runtime| {
                                let (used, reallocated, free): (Color, Color, Color) =
                                    theme.with(runtime, |theme| {
                                        (theme.text.muted, theme.accent.end, theme.control.track)
                                    });
                                vec![
                                    Segment::new(120.0, used),
                                    Segment::new(80.0, reallocated),
                                    Segment::new(300.0, free),
                                ]
                            })
                            .w(360.0),
                    )
                })
                .boxed()
            },
        },
        Preview {
            name: "segmented_bar_empty",
            build: || stage(segmented_bar([]).w(360.0)),
        },
    ]
}
