use crossbeam_channel::Receiver;
use semantics::SemanticId;

use crate::AccessibilityAction;

/// Something assistive technology did, delivered to the UI thread.
#[derive(Debug, Clone, PartialEq)]
pub enum AccessibilityEvent {
    /// A screen reader started listening; push a full tree with
    /// [`AccessibilityAdapter::update`](crate::AccessibilityAdapter::update).
    Activated,
    /// No assistive technology is listening anymore.
    Deactivated,
    /// Assistive technology asked for `action` on the node.
    Action(SemanticId, AccessibilityAction),
}

/// The UI thread's end of the adapter's event channel.
#[derive(Debug)]
pub struct AccessibilityEvents(pub(crate) Receiver<AccessibilityEvent>);

impl AccessibilityEvents {
    /// Every event that arrived since the last poll, without blocking.
    pub fn try_iter(&self) -> impl Iterator<Item = AccessibilityEvent> + '_ {
        self.0.try_iter()
    }
}
