use geometry::{CornerRadii, Insets, Point, Rect};
use graphics::Paint;
use reactive::{Runtime, Signal};
use view::{Cx, NodeId, Styled, View, bind};

use super::{handles::SelectHandles, metrics::*, value_label::value_label};
use crate::{
    canvas::canvas,
    kit::{Theme, icon::tinted_icon, icons, use_theme},
    stack::vstack,
    util::{interaction::Interaction, spring, themed::themed},
};

#[derive(Clone, Copy)]
enum BorderTone {
    Resting,
    Focused,
}

fn border_canvas(theme: Signal<Theme>, tone: BorderTone) -> impl Styled + View {
    canvas(move |runtime, bounds, list| {
        let border = theme.with(runtime, |theme| match tone {
            BorderTone::Resting => Paint::solid(theme.popover.border),
            BorderTone::Focused => theme.popover.ring.vertical(0.0, bounds.height),
        });
        let half = BORDER_WIDTH / 2.0;
        list.stroke(
            Rect::new(Point::ZERO, bounds).inset(Insets::uniform(half)),
            CornerRadii::uniform(TRIGGER_RADIUS - half),
            BORDER_WIDTH,
            border,
        );
    })
    .absolute()
    .inset(0.0)
}

/// The pill the trigger is drawn as: fill, a border that travels to the accent gradient on
/// focus, the value and the chevron.
pub(super) fn build_skin(
    cx: &mut Cx,
    root: NodeId,
    handles: SelectHandles,
    interaction: Interaction,
) -> NodeId {
    let theme = use_theme(cx);
    let disabled = cx.observe_disabled(root);
    let focused =
        move |runtime: &mut Runtime| interaction.is_focused(runtime) && !disabled.get(runtime);
    let chevron = tinted_icon(
        icons::CHEVRON_DOWN,
        ICON_STROKE,
        themed(theme, |theme| theme.popover.muted_text),
    )
    .size(CHEVRON_BOX)
    .shrink(0.0)
    .mr(CHEVRON_INSET);
    let content = vstack((value_label(cx, theme, disabled, handles), chevron))
        .absolute()
        .inset(0.0)
        .flex_row()
        .items_center();
    cx.build(
        vstack((
            border_canvas(theme, BorderTone::Resting),
            border_canvas(theme, BorderTone::Focused).opacity(bind(move |runtime| {
                if focused(runtime) { 1.0 } else { 0.0 }
            })),
            content,
        ))
        .absolute()
        .inset(0.0)
        .rounded(TRIGGER_RADIUS)
        .bg(themed(theme, |theme| theme.popover.trigger_bg))
        .transition(spring::snappy()),
    )
}
