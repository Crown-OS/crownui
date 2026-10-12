use accesskit::TreeUpdate;
use crossbeam_channel::{Receiver, Sender};
use semantics::{SemanticDiff, SemanticId, SemanticTree};

use crate::{
    AccessibilityEvents, Toolkit, TreeExporter,
    handlers::{ActionForwarder, DeactivationNotifier, InitialTreeProvider},
    util::ownership::Ownership,
};

/// Publishes a window's semantic tree over AT-SPI and relays assistive technology requests.
///
/// AccessKit runs its handlers on its own thread; they talk to this adapter and to the
/// returned [`AccessibilityEvents`] only through channels.
pub struct AccessibilityAdapter {
    adapter: accesskit_unix::Adapter,
    exporter: TreeExporter,
    full_trees: Sender<TreeUpdate>,
    superseded_full_trees: Receiver<TreeUpdate>,
    activations: Receiver<()>,
    needs_full_tree: bool,
    had_detached_nodes: bool,
}

impl AccessibilityAdapter {
    /// Registers a window with AccessKit. `app_name` labels the root window when its
    /// semantic node has no name; AT-SPI takes the application name from the executable.
    pub fn new(app_name: &str, toolkit: Toolkit) -> (Self, AccessibilityEvents) {
        let (event_sender, event_receiver) = crossbeam_channel::unbounded();
        let (full_tree_sender, full_tree_receiver) = crossbeam_channel::bounded(1);
        let (activation_sender, activation_receiver) = crossbeam_channel::unbounded();
        let adapter = accesskit_unix::Adapter::new(
            InitialTreeProvider::new(
                full_tree_receiver.clone(),
                activation_sender,
                event_sender.clone(),
            ),
            ActionForwarder::new(event_sender.clone()),
            DeactivationNotifier::new(event_sender),
        );
        let accessibility_adapter = Self {
            adapter,
            exporter: TreeExporter::new().window_label(app_name).toolkit(toolkit),
            full_trees: full_tree_sender,
            superseded_full_trees: full_tree_receiver,
            activations: activation_receiver,
            needs_full_tree: true,
            had_detached_nodes: false,
        };
        (accessibility_adapter, AccessibilityEvents(event_receiver))
    }

    /// Exports the whole tree: pushed to AccessKit while assistive technology listens,
    /// otherwise kept as the initial tree for the next activation.
    pub fn update(&mut self, tree: &SemanticTree, focus: Option<SemanticId>) {
        let ownership = Ownership::of(tree);
        self.had_detached_nodes = ownership.has_detached_nodes(tree);
        let delivered = self.push(|exporter, _| exporter.full_with(tree, &ownership, focus));
        if !delivered {
            self.superseded_full_trees.try_iter().for_each(drop);
            let _ = self
                .full_trees
                .try_send(self.exporter.full_with(tree, &ownership, focus));
        }
    }

    /// Exports only what `diff` changed, and builds nothing while no assistive
    /// technology listens. Sends a full tree instead after activation, a scale change, or
    /// while nodes are detached from the root.
    pub fn update_if_active(
        &mut self,
        diff: &SemanticDiff,
        tree_after: &SemanticTree,
        focus: Option<SemanticId>,
    ) {
        let had_detached_nodes = self.had_detached_nodes;
        let mut has_detached_nodes = had_detached_nodes;
        self.push(|exporter, needs_full_tree| {
            let ownership = Ownership::of(tree_after);
            has_detached_nodes = ownership.has_detached_nodes(tree_after);
            if needs_full_tree || had_detached_nodes || has_detached_nodes {
                exporter.full_with(tree_after, &ownership, focus)
            } else {
                exporter.incremental_with(diff, tree_after, &ownership, focus)
            }
        });
        self.had_detached_nodes = has_detached_nodes;
    }

    /// Reports whether the window has keyboard focus.
    pub fn set_focus_in_window(&mut self, is_focused: bool) {
        self.adapter.update_window_focus_state(is_focused);
    }

    /// Sets the window's device pixel ratio; the next update is a full one.
    pub fn set_scale_factor(&mut self, scale_factor: f64) {
        if self.exporter.set_scale_factor(scale_factor) {
            self.needs_full_tree = true;
        }
    }

    fn push(&mut self, build: impl FnOnce(&TreeExporter, bool) -> TreeUpdate) -> bool {
        let Self {
            adapter,
            exporter,
            activations,
            needs_full_tree,
            ..
        } = self;
        let mut delivered = false;
        adapter.update_if_active(|| {
            delivered = true;
            let activated = activations.try_iter().count() > 0;
            build(exporter, *needs_full_tree || activated)
        });
        if delivered {
            *needs_full_tree = false;
        }
        delivered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "needs a D-Bus session bus with AT-SPI"]
    fn adapter_registers_and_accepts_updates() {
        let (mut adapter, events) = AccessibilityAdapter::new("crownui-test", Toolkit::default());
        let tree = SemanticTree::new();
        adapter.set_focus_in_window(true);
        adapter.update(&tree, None);
        adapter.update_if_active(&SemanticDiff::default(), &tree, None);
        assert!(
            events
                .try_iter()
                .all(|event| event != crate::AccessibilityEvent::Deactivated)
        );
    }
}
