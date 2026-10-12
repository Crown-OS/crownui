use geometry::{CornerRadii, Rect, Vector};
use layouting::{Align, Dimension, Position};
use reactive::Signal;
use semantics::Role;
use view::{Cx, Kind, NodeId, SizeProperty, StyleProp, Styled, bind};
use windowing::CursorIcon;

use super::{
    input::{attach, track_width},
    metrics::{HEIGHT, MAX_WIDTH, MIN_WIDTH, THUMB_HALF_WIDTH, THUMB_OUTLINE_WIDTH, TRACK_HEIGHT},
    scale::SliderScale,
    thumb::thumb,
};
use crate::{canvas::canvas, kit::use_theme, stack::vstack, util::themed::themed};

/// The kind behind [`slider`](super::slider).
pub struct SliderKind {
    pub(super) value: Signal<f32>,
    pub(super) scale: SliderScale,
}

impl Kind for SliderKind {
    const NAME: &'static str = "kit_slider";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let (value, scale) = (self.value, self.scale);
        let size = cx.observe_size(node);

        cx.set_role(node, Role::Slider);
        for prop in [
            StyleProp::Position(Position::Relative),
            StyleProp::AlignSelf(Align::Start),
            StyleProp::Size(SizeProperty::Width, Dimension::Percent(1.0)),
            StyleProp::Size(SizeProperty::MinWidth, Dimension::Length(MIN_WIDTH)),
            StyleProp::Size(SizeProperty::MaxWidth, Dimension::Length(MAX_WIDTH)),
            StyleProp::Size(SizeProperty::Height, Dimension::Length(HEIGHT)),
            StyleProp::Shrink(1.0),
            StyleProp::Cursor(CursorIcon::Pointer),
        ] {
            cx.set_default_style(node, prop);
        }
        cx.bind_value(node, move |runtime| value.get(runtime).to_string());
        attach(cx, node, value, scale, size);

        let travel = move |runtime: &mut reactive::Runtime| {
            let progress = scale.progress(value.get(runtime));
            Vector::new(progress * track_width(size.get(runtime)), 0.0)
        };
        let accent = canvas(move |runtime, bounds, list| {
            let accent = theme.with(runtime, |theme| theme.accent);
            list.fill(
                Rect::new(geometry::Point::ZERO, bounds),
                CornerRadii::uniform(TRACK_HEIGHT / 2.0),
                accent.horizontal(0.0, bounds.width),
            );
        })
        .absolute()
        .inset(0.0);
        let unfilled = vstack(())
            .absolute()
            .inset(0.0)
            .bg(themed(theme, |theme| theme.control.track))
            .translate(bind(travel));
        let track = vstack((accent, unfilled))
            .absolute()
            .left(THUMB_HALF_WIDTH)
            .right(THUMB_HALF_WIDTH)
            .top((HEIGHT - TRACK_HEIGHT) / 2.0)
            .h(TRACK_HEIGHT)
            .rounded(TRACK_HEIGHT / 2.0)
            .overflow_hidden();

        let thumb = cx.build(thumb(theme, bind(travel)));
        cx.bind_style(thumb, move |runtime| {
            theme.with(runtime, |theme| {
                StyleProp::Ring(Some((THUMB_OUTLINE_WIDTH, theme.control.knob_shadow)))
            })
        });
        let track = cx.build(track);
        cx.adopt_children(node, vec![track, thumb]);
    }
}
