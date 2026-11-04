use reactive::Signal;
use view::{Cx, ElementTree, Listener, NodeId};

/// Keeps Tab and Shift+Tab cycling through the focusable nodes inside one panel.
#[derive(Clone, Copy)]
pub(super) struct FocusTrap {
    panel: NodeId,
    focused: Signal<Option<NodeId>>,
}

fn focusables(tree: &ElementTree, panel: NodeId) -> Vec<NodeId> {
    let mut found = Vec::new();
    let mut stack = vec![panel];
    while let Some(id) = stack.pop() {
        let Some(node) = tree.get(id) else {
            continue;
        };
        if node.visual().hidden {
            continue;
        }
        if id != panel && node.is_focusable() {
            found.push(id);
        }
        stack.extend(node.children().iter().rev().copied());
    }
    found
}

fn reading_order(tree: &ElementTree, mut nodes: Vec<NodeId>) -> Vec<NodeId> {
    let origin = |id: NodeId| {
        tree.get(id)
            .map(|node| node.geometry().window_rect.origin)
            .unwrap_or_default()
    };
    nodes.sort_by(|a, b| {
        let (a, b) = (origin(*a), origin(*b));
        a.y.total_cmp(&b.y).then(a.x.total_cmp(&b.x))
    });
    nodes
}

impl FocusTrap {
    /// Tracks focus inside `panel` and moves it to the first focusable node (or the panel).
    pub(super) fn install(cx: &mut Cx, panel: NodeId) -> Self {
        let focused = cx.signal(None);
        let inside = focusables(cx.tree(), panel);
        for node in inside.iter().copied().chain([panel]) {
            cx.add_listener(
                node,
                Listener::Focus(Box::new(move |cx, _| focused.set(cx, Some(node)))),
            );
        }
        cx.request_focus(inside.first().copied().unwrap_or(panel));
        Self { panel, focused }
    }

    /// Moves focus to the next (or previous) focusable node, wrapping around inside the panel.
    pub(super) fn cycle(self, cx: &mut Cx, backwards: bool) {
        let order = reading_order(cx.tree(), focusables(cx.tree(), self.panel));
        let Some(last) = order.len().checked_sub(1) else {
            cx.request_focus(self.panel);
            return;
        };
        let current = self
            .focused
            .get(cx)
            .and_then(|node| order.iter().position(|candidate| *candidate == node));
        let next = match (current, backwards) {
            (Some(0), true) | (None, true) => last,
            (Some(index), true) => index - 1,
            (Some(index), false) if index < last => index + 1,
            _ => 0,
        };
        cx.request_focus(order[next]);
    }
}
