use layouting::{Align, Dimension, Position};
use semantics::Role;
use view::{Cx, Kind, NodeId, Prop, SizeProperty, StyleProp, Styled};

use super::{
    axis::MeterAxis,
    metrics::{LENGTH, THICKNESS, VERTICAL_LENGTH},
    paint::paint_meter,
    reading::MeterReading,
};
use crate::{
    canvas::canvas,
    kit::use_theme,
    util::{prop_value::current, themed::bind_themed_style},
};

/// The kind behind [`level_meter`](super::level_meter).
pub struct LevelMeterKind {
    pub(super) peak: Prop<f32>,
    pub(super) rms: Option<Prop<f32>>,
    pub(super) axis: MeterAxis,
}

impl LevelMeterKind {
    fn default_size(&self) -> (f32, f32) {
        match self.axis {
            MeterAxis::Horizontal => (LENGTH, THICKNESS),
            MeterAxis::Vertical => (THICKNESS, VERTICAL_LENGTH),
        }
    }
}

impl Kind for LevelMeterKind {
    const NAME: &'static str = "kit_level_meter";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let (width, height) = self.default_size();
        let Self {
            mut peak,
            mut rms,
            axis,
        } = self;
        for prop in [
            StyleProp::Position(Position::Relative),
            StyleProp::AlignSelf(Align::Start),
            StyleProp::Shrink(0.0),
            StyleProp::Size(SizeProperty::Width, Dimension::Length(width)),
            StyleProp::Size(SizeProperty::Height, Dimension::Length(height)),
        ] {
            cx.set_default_style(node, prop);
        }

        let reading = cx.signal(MeterReading::default());
        cx.effect(move |runtime| {
            let peak = current(&mut peak, runtime);
            let rms = rms.as_mut().map(|rms| current(rms, runtime));
            reading.update(runtime, |reading| reading.follow(peak, rms));
        });
        let decibels = cx
            .memo(move |runtime| reading.with(runtime, MeterReading::peak_decibels).round() as i32);
        cx.set_role(node, Role::ProgressBar);
        cx.bind_value(node, move |runtime| format!("{} dB", decibels.get(runtime)));
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::Background(Some(theme.control.track))
        });

        let meter = canvas(move |runtime, size, list| {
            let reading = reading.get(runtime);
            theme.with(runtime, |theme| {
                paint_meter(reading, theme, axis, size, list)
            });
        })
        .absolute()
        .inset(0.0);
        cx.set_children(node, meter);
    }
}
