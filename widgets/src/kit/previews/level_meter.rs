use view::{Styled, View, component};

use crate::{
    kit::{
        LevelMeterOptions, level_meter,
        preview::{Preview, stage},
    },
    stack::hstack,
};

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "level_meter",
            build: || {
                stage((
                    level_meter(0.06),
                    level_meter(0.32).rms(0.12),
                    level_meter(0.7).rms(0.35),
                    level_meter(1.0).rms(0.6),
                ))
            },
        },
        Preview {
            name: "level_meter_hold",
            build: || {
                component(|cx| {
                    let peak = cx.signal(0.7);
                    cx.request_animation_frame(move |cx, _| peak.set(cx, 0.2));
                    stage((
                        level_meter(peak).rms(0.08),
                        hstack((
                            level_meter(0.05).vertical(),
                            level_meter(0.3).rms(0.12).vertical(),
                            level_meter(peak).vertical(),
                            level_meter(1.0).vertical(),
                        ))
                        .gap(12.0),
                    ))
                })
                .boxed()
            },
        },
    ]
}
