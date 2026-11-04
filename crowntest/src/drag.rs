use geometry::Point;
use windowing::{
    DndAction, DndActions, DragOffer, DragOutcome, DropAcceptance, DroppedData, WindowEvent,
};

use crate::TestApp;

/// Drags from other apps over the window, and the end of drags the app started.
impl TestApp {
    /// A drag offering `mime_types` and allowing `actions` enters the window at `(x, y)`.
    pub fn drag_enter(
        &mut self,
        x: f32,
        y: f32,
        mime_types: &[&str],
        actions: DndActions,
    ) -> &mut Self {
        self.deliver(WindowEvent::DragEnter(DragOffer {
            position: Point::new(x, y),
            mime_types: mime_types.iter().map(|mime| (*mime).to_owned()).collect(),
            actions,
        }))
    }

    /// The drag moves to `(x, y)`.
    pub fn drag_over(&mut self, x: f32, y: f32) -> &mut Self {
        self.deliver(WindowEvent::DragOver(Point::new(x, y)))
    }

    /// The drag leaves the window without dropping.
    pub fn drag_leave(&mut self) -> &mut Self {
        self.deliver(WindowEvent::DragLeave)
    }

    /// The drag drops `data` read as `mime_type` at `(x, y)`, the platform having settled on
    /// `action`.
    pub fn drop_data(
        &mut self,
        x: f32,
        y: f32,
        mime_type: &str,
        data: &[u8],
        action: DndAction,
    ) -> &mut Self {
        self.deliver(WindowEvent::Drop(DroppedData {
            position: Point::new(x, y),
            mime_type: mime_type.to_owned(),
            data: data.to_vec(),
            action,
        }))
    }

    /// How the window answered the drag over it, when that changed since the last call.
    pub fn drag_acceptance(&mut self) -> Option<Option<DropAcceptance>> {
        self.ui().take_drag_acceptance()
    }

    /// A drag the app started ends with `outcome`.
    pub fn end_drag(&mut self, outcome: DragOutcome) -> &mut Self {
        self.deliver(WindowEvent::DragEnded(outcome))
    }
}
