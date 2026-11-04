use std::time::Duration;

use reactive::{Signal, Store};
use view::{Cx, NodeId};

use super::metrics::{BUBBLE_HEIGHT, DELAY, EDGE_MARGIN, GAP};

/// Where the anchor sits in the window when the tooltip is revealed.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) struct AnchorFrame {
    /// The anchor's left edge, in window coordinates.
    pub(super) left: f32,
    /// The anchor's horizontal center, in window coordinates.
    pub(super) center: f32,
    /// The window's width.
    pub(super) window_width: f32,
    /// Whether the label goes above the anchor.
    pub(super) flipped: bool,
}

impl AnchorFrame {
    /// The label's left edge in window coordinates: centered under the anchor, kept inside the
    /// window.
    pub(super) fn label_left(self, width: f32) -> f32 {
        let centered = self.center - width / 2.0;
        let limit = (self.window_width - EDGE_MARGIN - width).max(EDGE_MARGIN);
        centered.clamp(EDGE_MARGIN, limit)
    }
}

/// Whether a tooltip shows and where, driven by the pointer resting on its anchor.
#[derive(Clone, Copy)]
pub(super) struct HoverReveal {
    pub(super) visible: Signal<bool>,
    pub(super) frame: Signal<AnchorFrame>,
    generation: Store<u64>,
    anchor: NodeId,
}

impl HoverReveal {
    pub(super) fn new(cx: &mut Cx, anchor: NodeId, visible: bool) -> Self {
        let reveal = Self {
            visible: cx.signal(false),
            frame: cx.signal(AnchorFrame::default()),
            generation: cx.store(0),
            anchor,
        };
        if visible {
            cx.request_animation_frame(move |cx, _| reveal.reveal(cx));
        }
        reveal
    }

    /// Shows the tooltip once the pointer has rested for the delay, unless it leaves first.
    pub(super) fn arm(self, cx: &mut Cx) {
        let ticket = self.generation.update(cx, |generation| {
            *generation += 1;
            *generation
        });
        self.wait(cx, ticket, None);
    }

    /// Hides the tooltip and cancels a pending reveal.
    pub(super) fn disarm(self, cx: &mut Cx) {
        self.generation.update(cx, |generation| *generation += 1);
        if self.visible.with_untracked(cx, |visible| *visible) {
            self.visible.set(cx, false);
        }
    }

    fn wait(self, cx: &mut Cx, ticket: u64, started: Option<Duration>) {
        cx.request_animation_frame(move |cx, now| {
            if self.generation.with(cx, |current| *current != ticket) {
                return;
            }
            let started = started.unwrap_or(now);
            if now.saturating_sub(started) >= DELAY {
                self.reveal(cx);
            } else {
                self.wait(cx, ticket, Some(started));
            }
        });
    }

    fn reveal(self, cx: &mut Cx) {
        let tree = cx.tree();
        let window = tree
            .root()
            .and_then(|root| tree.get(root))
            .map(|root| root.geometry().rect.size)
            .unwrap_or_default();
        if window.is_empty() {
            cx.request_animation_frame(move |cx, _| self.reveal(cx));
            return;
        }
        let anchor = tree
            .get(self.anchor)
            .map(|anchor| anchor.geometry().window_rect)
            .unwrap_or_default();
        let below = anchor.max_y() + GAP + BUBBLE_HEIGHT;
        let above = anchor.min_y() - GAP - BUBBLE_HEIGHT;
        let frame = AnchorFrame {
            left: anchor.min_x(),
            center: anchor.center().x,
            window_width: window.width,
            flipped: below > window.height && above >= 0.0,
        };
        self.frame.set(cx, frame);
        self.visible.set(cx, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(center: f32) -> AnchorFrame {
        AnchorFrame {
            left: center - 14.0,
            center,
            window_width: 400.0,
            flipped: false,
        }
    }

    #[test]
    fn a_label_centers_under_its_anchor_when_it_fits() {
        assert_eq!(frame(200.0).label_left(100.0), 150.0);
    }

    #[test]
    fn a_label_near_an_edge_slides_back_inside_the_window() {
        assert_eq!(frame(20.0).label_left(100.0), EDGE_MARGIN);
        assert_eq!(frame(390.0).label_left(100.0), 400.0 - EDGE_MARGIN - 100.0);
    }
}
