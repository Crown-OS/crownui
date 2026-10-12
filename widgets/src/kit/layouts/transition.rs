use geometry::{Size, Vector};
use layouting::Overflow;
use reactive::{Signal, Store};
use semantics::Role;
use view::{Cx, Element, Kind, NodeId, Semantic, StyleProp, Styled, View, for_each};

use crate::util::spring;

#[derive(Clone, PartialEq)]
struct Pane {
    serial: u64,
    ordinal: usize,
}

struct PaneKind<V> {
    content: V,
    previous: Store<Option<NodeId>>,
}

impl<V: View> Kind for PaneKind<V> {
    const NAME: &'static str = "slide_pane";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        cx.set_children(node, self.content);
        self.previous.update(cx, |previous| previous.replace(node));
    }
}

struct SlidePanes<F> {
    pane: F,
    previous_pane: Store<Option<NodeId>>,
    previous_ordinal: usize,
    container_size: Signal<Size>,
}

impl<F, V> SlidePanes<F>
where
    F: FnMut(&mut Cx, usize) -> V,
    V: View,
{
    fn row(&mut self, cx: &mut Cx, pane: &Pane) -> Element<PaneKind<V>> {
        let content = (self.pane)(cx, pane.ordinal);
        let travel = self.container_size.with_untracked(cx, |size| size.width);
        let direction = if pane.ordinal > self.previous_ordinal {
            1.0
        } else {
            -1.0
        };
        self.previous_ordinal = pane.ordinal;
        if let Some(leaving) = self.previous_pane.update(cx, |previous| previous.take()) {
            cx.set_default_style(
                leaving,
                StyleProp::Translate(Vector::new(-direction * travel, 0.0)),
            );
        }
        let incoming = Element::new(PaneKind {
            content,
            previous: self.previous_pane,
        })
        .flex_1()
        .exit(|style| style);
        if pane.serial == 0 {
            incoming
        } else {
            incoming.enter(|style| style.translate_x(direction * travel))
        }
    }
}

/// The kind behind [`slide_transition`].
pub struct SlideTransitionKind<F> {
    index: Signal<usize>,
    pane: F,
}

impl<F, V> Kind for SlideTransitionKind<F>
where
    F: FnMut(&mut Cx, usize) -> V + 'static,
    V: View,
{
    const NAME: &'static str = "kit_slide_transition";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        cx.set_default_style(node, StyleProp::Overflow(Overflow::Clip));
        let Self { index, pane } = self;
        let first = index.with_untracked(cx, |ordinal| *ordinal);
        let panes = cx.signal(vec![Pane {
            serial: 0,
            ordinal: first,
        }]);
        cx.effect(move |runtime| {
            let ordinal = index.get(runtime);
            panes.update(runtime, |panes| {
                if let Some(current) = panes
                    .first_mut()
                    .filter(|current| current.ordinal != ordinal)
                {
                    *current = Pane {
                        serial: current.serial + 1,
                        ordinal,
                    };
                }
            });
        });
        let mut slide = SlidePanes {
            pane,
            previous_pane: cx.store(None),
            previous_ordinal: first,
            container_size: cx.observe_size(node),
        };
        let strip = for_each(
            panes,
            |pane: &Pane| pane.serial,
            move |cx, pane: Signal<Pane>| {
                let pane = pane.with_untracked(cx, Clone::clone);
                slide.row(cx, &pane)
            },
        )
        .role(Role::Group)
        .flex_1();
        cx.set_children(node, strip);
    }
}

/// Slides between the panes `pane(cx, ordinal)` builds as `index` changes: the new pane enters
/// from the right when the ordinal grew and from the left when it shrank, the old one leaves by
/// the opposite edge, both clipped to the container. The old pane is retired when its exit
/// finishes, so the caller never manages an outgoing pane. Panes grow to fill the container, so a
/// pane holding a `scroll_view` scrolls within it.
pub fn slide_transition<F, V>(index: Signal<usize>, pane: F) -> Element<SlideTransitionKind<F>>
where
    F: FnMut(&mut Cx, usize) -> V + 'static,
    V: View,
{
    Element::new(SlideTransitionKind { index, pane }).transition(spring::snappy())
}
