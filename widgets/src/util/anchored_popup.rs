use geometry::{Rect, Size};
use motion::Curve;
use reactive::{Runtime, ScopeId, Signal, Store};
use view::{Cx, Element, Kind, NodeId, StyleProp, Styled};

use super::spring;

pub(crate) const PANEL_GAP: f32 = 5.0;
pub(crate) const OVERLAY_Z_INDEX: i16 = 100;
const APPEAR_SCALE: f32 = 0.95;
const APPEAR_SLIDE: f32 = 8.0;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Placement {
    top: f32,
    flipped: bool,
    lane_left: f32,
    lane_width: f32,
}

fn place_panel(
    anchor: Rect,
    anchor_size: Size,
    window_height: f32,
    panel_height: f32,
) -> Placement {
    let laid_out = !anchor.size.is_empty();
    let below = anchor.min_y() + anchor_size.height + PANEL_GAP;
    let above = anchor.min_y() - PANEL_GAP - panel_height;
    let flipped = laid_out && below + panel_height > window_height && above >= 0.0;
    Placement {
        top: if flipped {
            -(panel_height + PANEL_GAP)
        } else {
            anchor_size.height + PANEL_GAP
        },
        flipped,
        lane_left: if laid_out { -anchor.min_x() } else { 0.0 },
        lane_width: if laid_out {
            anchor.max_x()
        } else {
            anchor_size.width
        },
    }
}

/// A curve that settles in a single frame, for descendants that must not follow an ancestor's spring.
pub(crate) fn instant() -> Curve {
    Curve::linear(0)
}

/// Open state and placement of a panel that unfolds from an anchor node, flipping above it when
/// there is no room below.
#[derive(Clone, Copy)]
pub(crate) struct AnchoredPopup {
    anchor: NodeId,
    anchor_size: Size,
    open: Signal<bool>,
    placement: Signal<Placement>,
    raised: Store<bool>,
    scope: ScopeId,
}

impl AnchoredPopup {
    pub(crate) fn new(cx: &mut Cx, anchor: NodeId, anchor_size: Size) -> Self {
        Self {
            anchor,
            anchor_size,
            open: cx.signal(false),
            placement: cx.signal(Placement::default()),
            raised: cx.store(false),
            scope: cx.current_scope(),
        }
    }

    pub(crate) fn open_signal(self) -> Signal<bool> {
        self.open
    }

    pub(crate) fn is_open(self, runtime: &mut Runtime) -> bool {
        self.open.get(runtime)
    }

    pub(crate) fn top(self, runtime: &mut Runtime) -> f32 {
        self.placement.with(runtime, |placement| placement.top)
    }

    /// The horizontal band from the window's left edge to the anchor's right edge, as an offset
    /// from the anchor and a width: a panel pushed to the band's end hangs off the anchor's
    /// trailing edge and stays inside the window.
    pub(crate) fn trailing_lane(self, runtime: &mut Runtime) -> (f32, f32) {
        self.placement.with(runtime, |placement| {
            (placement.lane_left, placement.lane_width)
        })
    }

    pub(crate) fn reveal(self, cx: &mut Cx, panel_height: f32) {
        let tree = cx.tree();
        let window_height = tree
            .root()
            .and_then(|root| tree.get(root))
            .map_or(f32::INFINITY, |root| root.geometry().rect.size.height);
        let anchor = tree
            .get(self.anchor)
            .map_or(Rect::default(), |anchor| anchor.geometry().window_rect);
        let placement = place_panel(anchor, self.anchor_size, window_height, panel_height);
        self.placement.set(cx, placement);
        self.raise_ancestors(cx);
        self.open.set(cx, true);
    }

    fn raise_ancestors(self, cx: &mut Cx) {
        if self
            .raised
            .update(cx, |bound| std::mem::replace(bound, true))
        {
            return;
        }
        let tree = cx.tree();
        let previous: Vec<(NodeId, i16)> = tree
            .ancestors(self.anchor)
            .filter_map(|node| Some((node, tree.get(node)?.visual().z_index)))
            .collect();
        let outer_scope = cx.set_current_scope(self.scope);
        for (node, resting) in previous {
            let open = self.open;
            cx.bind_style(node, move |runtime| {
                let raised = open.try_get(runtime).unwrap_or(false);
                StyleProp::ZIndex(if raised { OVERLAY_Z_INDEX } else { resting })
            });
        }
        cx.set_current_scope(outer_scope);
    }

    pub(crate) fn dismiss(self, runtime: &mut Runtime) {
        if self.open.with_untracked(runtime, |open| *open) {
            self.open.set(runtime, false);
        }
    }

    pub(crate) fn toggle(self, cx: &mut Cx, panel_height: f32) {
        if self.open.get(cx) {
            self.dismiss(cx);
        } else {
            self.reveal(cx, panel_height);
        }
    }

    /// Fades and zooms `panel` in from, and back into, the edge it hangs off.
    pub(crate) fn appear<K: Kind>(self, cx: &Cx, panel: Element<K>) -> Element<K> {
        let flipped = self
            .placement
            .with_untracked(cx, |placement| placement.flipped);
        let slide = if flipped { APPEAR_SLIDE } else { -APPEAR_SLIDE };
        panel
            .origin(0.5, if flipped { 1.0 } else { 0.0 })
            .enter(|style| style.opacity(0.0).scale(APPEAR_SCALE).translate_y(slide))
            .exit(|style| style.opacity(0.0).scale(APPEAR_SCALE).translate_y(slide))
            .transition(spring::crisp())
    }
}

#[cfg(test)]
mod tests {
    use geometry::Point;

    use super::*;

    const ANCHOR_SIZE: Size = Size::new(100.0, 34.0);

    fn anchor_at(x: f32, y: f32) -> Rect {
        Rect::new(Point::new(x, y), ANCHOR_SIZE)
    }

    #[test]
    fn a_panel_with_room_hangs_below_the_anchor_after_the_gap() {
        let placement = place_panel(anchor_at(10.0, 20.0), ANCHOR_SIZE, 400.0, 160.0);
        assert_eq!((placement.top, placement.flipped), (39.0, false));
    }

    #[test]
    fn a_panel_without_room_below_flips_above_when_there_is_room_above() {
        let placement = place_panel(anchor_at(10.0, 300.0), ANCHOR_SIZE, 400.0, 160.0);
        assert_eq!((placement.top, placement.flipped), (-165.0, true));
    }

    #[test]
    fn a_panel_with_no_room_either_way_stays_below() {
        let placement = place_panel(anchor_at(10.0, 20.0), ANCHOR_SIZE, 100.0, 160.0);
        assert!(!placement.flipped);
    }

    #[test]
    fn the_trailing_lane_runs_from_the_window_edge_to_the_anchors_right_edge() {
        let placement = place_panel(anchor_at(30.0, 20.0), ANCHOR_SIZE, 400.0, 160.0);
        assert_eq!((placement.lane_left, placement.lane_width), (-30.0, 130.0));
    }

    #[test]
    fn an_anchor_that_has_not_been_laid_out_gets_a_lane_as_wide_as_itself() {
        let placement = place_panel(Rect::default(), ANCHOR_SIZE, 400.0, 160.0);
        assert_eq!((placement.lane_left, placement.lane_width), (0.0, 100.0));
        assert!(!placement.flipped);
    }
}
