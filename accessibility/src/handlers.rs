use accesskit::{ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, TreeUpdate};
use crossbeam_channel::{Receiver, Sender};

use crate::{AccessibilityEvent, action_request_to_ui};

/// Answers AccessKit's initial-tree request from the latest full tree the UI thread sent.
pub(crate) struct InitialTreeProvider {
    latest_tree: Option<TreeUpdate>,
    full_trees: Receiver<TreeUpdate>,
    activations: Sender<()>,
    events: Sender<AccessibilityEvent>,
}

impl InitialTreeProvider {
    pub(crate) fn new(
        full_trees: Receiver<TreeUpdate>,
        activations: Sender<()>,
        events: Sender<AccessibilityEvent>,
    ) -> Self {
        Self {
            latest_tree: None,
            full_trees,
            activations,
            events,
        }
    }
}

impl ActivationHandler for InitialTreeProvider {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        if let Some(newest) = self.full_trees.try_iter().last() {
            self.latest_tree = Some(newest);
        }
        let _ = self.activations.send(());
        let _ = self.events.send(AccessibilityEvent::Activated);
        self.latest_tree.clone()
    }
}

/// Forwards AccessKit action requests to the UI thread.
pub(crate) struct ActionForwarder {
    events: Sender<AccessibilityEvent>,
}

impl ActionForwarder {
    pub(crate) fn new(events: Sender<AccessibilityEvent>) -> Self {
        Self { events }
    }
}

impl ActionHandler for ActionForwarder {
    fn do_action(&mut self, request: ActionRequest) {
        if let Some((target, action)) = action_request_to_ui(&request) {
            let _ = self.events.send(AccessibilityEvent::Action(target, action));
        }
    }
}

/// Tells the UI thread that assistive technology stopped listening.
pub(crate) struct DeactivationNotifier {
    events: Sender<AccessibilityEvent>,
}

impl DeactivationNotifier {
    pub(crate) fn new(events: Sender<AccessibilityEvent>) -> Self {
        Self { events }
    }
}

impl DeactivationHandler for DeactivationNotifier {
    fn deactivate_accessibility(&mut self) {
        let _ = self.events.send(AccessibilityEvent::Deactivated);
    }
}
