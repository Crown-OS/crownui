use accessibility::{
    AccessibilityAction, AccessibilityAdapter, AccessibilityEvent, AccessibilityEvents, Toolkit,
};
use view::Ui;

/// Keeps screen readers in sync with the semantic tree and forwards their actions.
pub(crate) struct AccessibilityBridge {
    adapter: AccessibilityAdapter,
    events: AccessibilityEvents,
}

impl AccessibilityBridge {
    pub(crate) fn start(title: &str, ui: &mut Ui) -> Self {
        let (mut adapter, events) = AccessibilityAdapter::new(title, Toolkit::default());
        let focus = ui.focused_semantic_id();
        adapter.update(ui.semantic_tree(), focus);
        Self { adapter, events }
    }

    pub(crate) fn poll(&mut self, ui: &mut Ui) {
        let events: Vec<AccessibilityEvent> = self.events.try_iter().collect();
        for event in events {
            match event {
                AccessibilityEvent::Activated => {
                    let focus = ui.focused_semantic_id();
                    self.adapter.update(ui.semantic_tree(), focus);
                }
                AccessibilityEvent::Deactivated => {}
                AccessibilityEvent::Action(id, AccessibilityAction::Ui(action)) => {
                    if let Err(error) = ui.perform(id, &action) {
                        tracing::warn!("accessibility action refused: {error}");
                    }
                }
                AccessibilityEvent::Action(id, other) => {
                    tracing::debug!(?id, ?other, "accessibility action not supported yet");
                }
            }
        }
    }

    pub(crate) fn after_frame(&mut self, ui: &mut Ui) {
        if let Some(diff) = ui.take_semantic_diff() {
            let focus = ui.focused_semantic_id();
            let tree = ui.semantic_tree().clone();
            self.adapter.update_if_active(&diff, &tree, focus);
        }
    }

    pub(crate) fn set_scale(&mut self, scale: f64) {
        self.adapter.set_scale_factor(scale);
    }

    pub(crate) fn set_window_focused(&mut self, focused: bool) {
        self.adapter.set_focus_in_window(focused);
    }
}
