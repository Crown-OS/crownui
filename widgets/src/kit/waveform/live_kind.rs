use std::iter;

use reactive::Signal;
use semantics::Role;
use view::{Cx, Kind, NodeId, Styled};

use super::{
    bar::{bar_count, paint_bar},
    layout::set_default_size,
    level_history::LevelHistory,
    metrics::{BAR_PITCH, BAR_WIDTH},
};
use crate::{canvas::canvas, kit::use_theme, util::color::mix};

/// The kind behind [`live_waveform`](super::live_waveform).
pub struct LiveWaveformKind {
    pub(super) history: Signal<LevelHistory>,
}

impl Kind for LiveWaveformKind {
    const NAME: &'static str = "kit_live_waveform";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let history = self.history;
        set_default_size(cx, node);
        cx.set_role(node, Role::Image);

        let bars = canvas(move |runtime, size, list| {
            let accent = theme.with(runtime, |theme| theme.accent);
            history.with(runtime, |history| {
                let count = bar_count(size.width).min(LevelHistory::CAPACITY);
                let levels = history.newest_first().chain(iter::repeat(0.0));
                for (age, level) in levels.take(count).enumerate() {
                    let x = size.width - BAR_WIDTH - age as f32 * BAR_PITCH;
                    let color = mix(accent.start, accent.end, x / size.width);
                    paint_bar(list, x, level, size, color);
                }
            });
        })
        .absolute()
        .inset(0.0);
        cx.set_children(node, bars);
    }
}
