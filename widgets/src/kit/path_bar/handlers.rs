use reactive::Store;
use view::{Cx, DragOverEvent, DropEvent, Event};

pub(super) type SelectHandler = Box<dyn FnMut(&mut Cx, usize)>;
pub(super) type DragOverHandler = Box<dyn FnMut(&mut Cx, usize, &mut Event<DragOverEvent>)>;
pub(super) type DropHandler = Box<dyn FnMut(&mut Cx, usize, &mut Event<DropEvent>)>;

/// What a path bar runs when a segment is chosen or a drag reaches one.
#[derive(Default)]
pub(super) struct SegmentHandlers {
    pub(super) select: Option<SelectHandler>,
    pub(super) drag_over: Option<DragOverHandler>,
    pub(super) drop: Option<DropHandler>,
}

/// Runs one stored handler with the context it needs, putting it back afterwards.
pub(super) fn run<H>(
    cx: &mut Cx,
    handlers: Store<SegmentHandlers>,
    slot: fn(&mut SegmentHandlers) -> &mut Option<H>,
    call: impl FnOnce(&mut Cx, &mut H),
) {
    let Some(mut handler) = handlers.update(cx, |all| slot(all).take()) else {
        return;
    };
    call(cx, &mut handler);
    handlers.update(cx, |all| {
        slot(all).get_or_insert(handler);
    });
}
