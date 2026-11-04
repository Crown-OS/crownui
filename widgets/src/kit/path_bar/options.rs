use view::{Cx, DragOverEvent, DropEvent, Element, Event};

use super::kind::PathBarKind;

/// The options of a [`path_bar`](super::path_bar), chained before its modifiers.
pub trait PathBarOptions: Sized {
    /// Runs while a drag moves over segment `index` (HTML `dragover`); call `event.accept(..)`
    /// to take the drop there, and the segment lights up.
    fn on_segment_drag_over(
        self,
        handler: impl FnMut(&mut Cx, usize, &mut Event<DragOverEvent>) + 'static,
    ) -> Self;

    /// Runs when a drag the segment accepted is dropped on segment `index` (HTML `drop`).
    fn on_segment_drop(
        self,
        handler: impl FnMut(&mut Cx, usize, &mut Event<DropEvent>) + 'static,
    ) -> Self;
}

impl PathBarOptions for Element<PathBarKind> {
    fn on_segment_drag_over(
        self,
        handler: impl FnMut(&mut Cx, usize, &mut Event<DragOverEvent>) + 'static,
    ) -> Self {
        self.configure(|kind| kind.handlers.drag_over = Some(Box::new(handler)))
    }

    fn on_segment_drop(
        self,
        handler: impl FnMut(&mut Cx, usize, &mut Event<DropEvent>) + 'static,
    ) -> Self {
        self.configure(|kind| kind.handlers.drop = Some(Box::new(handler)))
    }
}
