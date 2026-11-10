use geometry::{CornerRadii, Point, Rect};
use reactive::{Runtime, Signal};
use view::{AnyView, Element, Styled, View, ViewSeq};

use super::metrics::ICON_COLUMN;
use crate::{
    canvas::canvas,
    kit::Theme,
    stack::{StackKind, hstack},
    util::surface_paint::SurfacePaint,
};

/// How a disc behind a glyph is finished in a theme.
pub(super) type DiscPaint = fn(&Theme) -> SurfacePaint;

fn column<C: ViewSeq>(content: C) -> Element<StackKind<C>> {
    hstack(content)
        .size(ICON_COLUMN)
        .shrink(0.0)
        .items_center()
        .justify_center()
}

fn centred(glyph: AnyView, size: f32) -> impl View {
    hstack(glyph).size(size).items_center().justify_center()
}

/// `glyph`, `size` square, bare in a row's icon column.
pub(super) fn plain(glyph: AnyView, size: f32) -> Element<StackKind<impl ViewSeq>> {
    column(centred(glyph, size))
}

/// `glyph`, `size` square, on the disc `disc` picks, filling a row's icon column.
pub(super) fn on_disc(
    theme: Signal<Theme>,
    glyph: AnyView,
    size: f32,
    mut disc: impl FnMut(&mut Runtime) -> DiscPaint + 'static,
) -> Element<StackKind<impl ViewSeq>> {
    let backdrop = canvas(move |runtime, bounds, list| {
        let pick = disc(runtime);
        theme.with(runtime, pick).paint(
            list,
            Rect::new(Point::ZERO, bounds),
            CornerRadii::uniform(bounds.height.min(bounds.width) / 2.0),
        );
    })
    .absolute()
    .inset(0.0);
    column((backdrop, centred(glyph, size))).relative()
}
