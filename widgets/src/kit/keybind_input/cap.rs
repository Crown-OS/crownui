use geometry::{CornerRadii, Point, Rect};
use reactive::{Runtime, Signal};
use view::{Styled, View, bind};

use super::{
    metrics::{
        CAP_BORDER_ALPHA, CAP_FILL_ALPHA, CAP_HEIGHT, CAP_PADDING, CAP_RADIUS, CAP_TEXT_SIZE,
    },
    recorder::Piece,
};
use crate::{canvas::canvas, kit::Theme, stack::hstack, text::text, util::color::scale_alpha};

pub(super) fn piece(theme: Signal<Theme>, disabled: Signal<bool>, piece: Piece) -> view::AnyView {
    match piece {
        Piece::Cap(label) => cap(theme, disabled, label).boxed(),
        Piece::Hint(hint) => text(hint)
            .shrink(0.0)
            .text_color(bind(move |runtime: &mut Runtime| {
                let disabled = disabled.get(runtime);
                theme.with(runtime, |theme| {
                    if disabled {
                        theme.text.disabled
                    } else {
                        theme.popover.muted_text
                    }
                })
            }))
            .boxed(),
    }
}

fn cap(theme: Signal<Theme>, disabled: Signal<bool>, label: String) -> impl View {
    let ink = move |runtime: &mut Runtime| {
        let disabled = disabled.get(runtime);
        theme.with(runtime, |theme| {
            if disabled {
                theme.text.disabled
            } else {
                theme.popover.text
            }
        })
    };
    let plate = canvas(move |runtime, bounds, list| {
        let ink = ink(runtime);
        let rect = Rect::new(Point::ZERO, bounds);
        let radii = CornerRadii::uniform(CAP_RADIUS);
        list.fill(rect, radii, scale_alpha(ink, CAP_FILL_ALPHA));
        list.stroke(rect, radii, 1.0, scale_alpha(ink, CAP_BORDER_ALPHA));
    })
    .absolute()
    .inset(0.0);
    hstack((
        plate,
        text(label)
            .text_size(CAP_TEXT_SIZE)
            .text_color(bind(ink))
            .shrink(0.0),
    ))
    .px(CAP_PADDING)
    .h(CAP_HEIGHT)
    .items_center()
    .relative()
    .shrink(0.0)
}
