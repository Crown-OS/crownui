use view::{AnyView, Styled, View, ViewSeq, component};

use crate::{
    kit::{
        LevelHistory, WaveformOptions, WaveformPeaks, live_waveform,
        preview::{Preview, stage},
        use_theme, waveform,
    },
    stack::vstack,
    util::themed::themed,
};

const SPEECH_BUCKETS: usize = 240;

fn speech_like(moment: usize) -> f32 {
    let time = moment as f32;
    let syllable = (time * 0.31).sin().abs();
    let phrase = ((time * 0.045).sin() * 0.5 + 0.5).powf(0.6);
    let breath = if (time * 0.021).sin() > 0.85 {
        0.1
    } else {
        1.0
    };
    (syllable * phrase * breath).clamp(0.0, 1.0)
}

fn recorded_peaks() -> WaveformPeaks {
    let levels: Vec<f32> = (0..SPEECH_BUCKETS).map(speech_like).collect();
    WaveformPeaks::from_levels(&levels)
}

fn on_surface(content: impl ViewSeq + 'static) -> AnyView {
    component(|cx| {
        let theme = use_theme(cx);
        stage(
            vstack(content)
                .w(432.0)
                .p(16.0)
                .gap(16.0)
                .rounded(12.0)
                .bg(themed(theme, |theme| theme.surface.bg)),
        )
    })
    .boxed()
}

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "waveform",
            build: || {
                on_surface((
                    waveform(recorded_peaks(), 0.0),
                    waveform(recorded_peaks(), 0.38).on_seeking(|_, _| {}),
                ))
            },
        },
        Preview {
            name: "live_waveform",
            build: || {
                on_surface(component(|cx| {
                    let mut history = LevelHistory::new();
                    (0..LevelHistory::CAPACITY)
                        .for_each(|moment| history.push(speech_like(moment * 3)));
                    live_waveform(cx.signal(history))
                }))
            },
        },
    ]
}
