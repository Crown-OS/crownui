use geometry::Point;
use windowing::{DragOffer, DroppedData};

use crate::{
    NodeId,
    event::{DragLeaveEvent, DragOverEvent, DropEvent, Event},
    handler::Dispatchable,
    ui::Ui,
};

/// The drag over the window, as the elements under it see it.
#[derive(Default)]
pub(crate) struct DropHover {
    path: Vec<NodeId>,
    mime_types: Vec<String>,
    accepted: Option<String>,
    unanswered: bool,
}

impl Ui {
    pub(crate) fn drag_enter(&mut self, offer: &DragOffer) {
        self.drop_hover.mime_types.clone_from(&offer.mime_types);
        self.drop_hover.accepted = None;
        self.drag_over(offer.position);
    }

    /// Fires `dragleave` on elements the drag moved off and `dragover` along the path under it,
    /// then records which MIME type, if any, the handlers accepted.
    pub(crate) fn drag_over(&mut self, position: Point) {
        let path = self.cx.tree.hit_test(position);
        let previous = std::mem::take(&mut self.drop_hover.path);
        for node in previous.iter().filter(|node| !path.contains(node)) {
            self.fire_drag_leave(*node);
        }
        let detail = DragOverEvent::new(position, self.drop_hover.mime_types.clone());
        let mut event = Dispatchable::DragOver(Event::new(detail, false));
        self.dispatch_along(&path, &mut event);
        let accepted = match &event {
            Dispatchable::DragOver(event) => event.accepted().map(str::to_owned),
            _ => None,
        };
        self.drop_hover.unanswered |= accepted != self.drop_hover.accepted;
        self.drop_hover.accepted = accepted;
        self.drop_hover.path = path;
    }

    pub(crate) fn drag_leave(&mut self) {
        for node in std::mem::take(&mut self.drop_hover.path) {
            self.fire_drag_leave(node);
        }
        self.drop_hover.accepted = None;
    }

    pub(crate) fn drop_data(&mut self, dropped: &DroppedData) {
        self.drop_hover.path.clear();
        self.drop_hover.accepted = None;
        let path = self.cx.tree.hit_test(dropped.position);
        let detail = DropEvent {
            position: dropped.position,
            mime_type: dropped.mime_type.clone(),
            data: dropped.data.clone(),
        };
        self.dispatch_along(&path, &mut Dispatchable::Drop(Event::new(detail, false)));
    }

    fn fire_drag_leave(&mut self, node: NodeId) {
        let mut event = Dispatchable::DragLeave(Event::new(DragLeaveEvent, false));
        self.dispatch_along(&[node], &mut event);
    }

    /// The MIME type the drag over the window is accepted as (`None` refuses it), when that
    /// answer changed since the last call.
    pub fn take_drag_acceptance(&mut self) -> Option<Option<String>> {
        std::mem::take(&mut self.drop_hover.unanswered).then(|| self.drop_hover.accepted.clone())
    }

    pub(crate) fn forget_drop_nodes(&mut self, removed: &[NodeId]) {
        self.drop_hover.path.retain(|node| !removed.contains(node));
    }
}
