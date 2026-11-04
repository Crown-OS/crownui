use geometry::{CornerRadii, Point, Rect, Size};
use semantics::Role;
use view::{Cx, Kind, NodeId, Prop, StyleProp, Styled};
use windowing::CursorIcon;

use super::{
    bar::{bar_count, paint_bar},
    layout::set_default_size,
    metrics::{BAR_WIDTH, PLAYHEAD_WIDTH},
    peaks::WaveformPeaks,
    seek::{Seek, SeekHandler},
};
use crate::{
    canvas::canvas,
    kit::use_theme,
    util::{color::mix, shared_prop::shared_prop, unit_interval::clamp_unit},
};

/// The kind behind [`waveform`](super::waveform).
pub struct WaveformKind {
    pub(super) peaks: Prop<WaveformPeaks>,
    pub(super) progress: Prop<f32>,
    pub(super) on_seeking: Option<SeekHandler>,
}

impl Kind for WaveformKind {
    const NAME: &'static str = "kit_waveform";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let peaks = shared_prop(cx, self.peaks);
        let progress = shared_prop(cx, self.progress.map(clamp_unit));
        set_default_size(cx, node);
        cx.bind_value(node, move |runtime| {
            format!("{}%", (progress.get(runtime) * 100.0).round())
        });
        match self.on_seeking {
            Some(handler) => {
                cx.set_role(node, Role::Slider);
                cx.set_default_style(node, StyleProp::Cursor(CursorIcon::Pointer));
                Seek {
                    handler: cx.store(Some(handler)),
                    size: cx.observe_size(node),
                    progress,
                }
                .attach(cx, node);
            }
            None => cx.set_role(node, Role::ProgressBar),
        }

        let bars = canvas(move |runtime, size, list| {
            let playhead = progress.get(runtime) * size.width;
            let (accent, unplayed, head) = theme.with(runtime, |theme| {
                (theme.accent, theme.text.muted, theme.text.primary)
            });
            peaks.with(runtime, |peaks| {
                let count = bar_count(size.width);
                let spread = (size.width - BAR_WIDTH) / count.saturating_sub(1).max(1) as f32;
                for bar in 0..count {
                    let x = bar as f32 * spread;
                    let color = if x + BAR_WIDTH / 2.0 <= playhead {
                        mix(accent.start, accent.end, x / size.width)
                    } else {
                        unplayed
                    };
                    paint_bar(list, x, peaks.bar_level(bar, count), size, color);
                }
            });
            let left = (playhead - PLAYHEAD_WIDTH / 2.0)
                .clamp(0.0, (size.width - PLAYHEAD_WIDTH).max(0.0));
            list.fill(
                Rect::new(
                    Point::new(left, 0.0),
                    Size::new(PLAYHEAD_WIDTH, size.height),
                ),
                CornerRadii::uniform(PLAYHEAD_WIDTH / 2.0),
                head,
            );
        })
        .absolute()
        .inset(0.0);
        cx.set_children(node, bars);
    }
}
