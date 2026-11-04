use geometry::Point;
use input::{Key, NamedKey};
use reactive::Signal;
use view::{
    Cx, Edges, Element, Kind, Listener, NodeId, StyleProp, Styled, ViewSeq, bind, component, show,
};

use super::{container::context_menu, placement::place_at};
use crate::{
    stack::vstack,
    util::window_cover::WindowCover,
    util::{anchored_popup::instant, spring},
};

const APPEAR_SCALE: f32 = 0.95;

/// The kind behind [`context_menu_at`].
pub struct ContextMenuAtKind<F> {
    position: Signal<Option<Point>>,
    items: F,
}

/// The kind behind the window-covering layer of an open [`context_menu_at`]: it catches presses
/// outside the menu and places the menu at the pointer.
pub struct ContextMenuLayerKind<Items> {
    position: Signal<Option<Point>>,
    cover: WindowCover,
    items: Items,
}

impl<Items: ViewSeq> Kind for ContextMenuLayerKind<Items> {
    const NAME: &'static str = "kit_context_menu_layer";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let Self {
            position,
            cover,
            items,
        } = self;
        let close = move |cx: &mut Cx| position.set(cx, None);
        let panel = cx.build(
            context_menu(items)
                .absolute()
                .origin(0.0, 0.0)
                .enter(|style| style.opacity(0.0).scale(APPEAR_SCALE))
                .transition(spring::crisp()),
        );
        let listeners = [
            Listener::Click(Box::new(move |cx, _| close(cx))),
            Listener::PointerDown(Box::new(|_, event| event.stop_propagation())),
            Listener::KeyDown(Box::new(move |cx, event| {
                if event.key == Key::Named(NamedKey::Escape) {
                    close(cx);
                    event.prevent_default();
                    event.stop_propagation();
                }
            })),
        ];
        for listener in listeners {
            cx.add_listener(panel, listener);
        }
        let panel_size = cx.observe_size(panel);
        let placed = move |runtime: &mut reactive::Runtime| {
            let point = position.get(runtime).unwrap_or(Point::ZERO);
            place_at(point, panel_size.get(runtime), cover.size(runtime))
        };
        cx.bind_style(panel, move |runtime| {
            StyleProp::Inset(Edges::Left, placed(runtime).x)
        });
        cx.bind_style(panel, move |runtime| {
            StyleProp::Inset(Edges::Top, placed(runtime).y)
        });
        cx.add_listener(
            node,
            Listener::PointerDown(Box::new(move |cx, _| close(cx))),
        );
        cx.adopt_children(node, vec![panel]);
        cx.request_focus(panel);
    }
}

impl<F, Items> Kind for ContextMenuAtKind<F>
where
    F: FnMut() -> Items + 'static,
    Items: ViewSeq,
{
    const NAME: &'static str = "kit_context_menu_at";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let Self {
            position,
            mut items,
        } = self;
        cx.set_default_style(node, StyleProp::Position(layouting::Position::Absolute));
        cx.set_children(
            node,
            show(
                bind(move |runtime| position.with(runtime, Option::is_some)),
                move || {
                    let items = items();
                    component(move |cx| {
                        let cover = WindowCover::track(cx, node);
                        cover.fill(
                            Element::new(ContextMenuLayerKind {
                                position,
                                cover,
                                items,
                            })
                            .exit(|style| style.opacity(0.0))
                            .transition(instant()),
                        )
                    })
                },
                || vstack(()),
            ),
        );
    }
}

/// Opens a [`context_menu`] of `items()` where `position` says, in window coordinates (set it
/// from an `on_context_menu` handler), above everything in the window. It flips left or up
/// rather than cross the window's edge, and sets `position` to `None` on a press outside it,
/// on Escape and on a click on any of its rows.
pub fn context_menu_at<F, Items>(
    position: Signal<Option<Point>>,
    items: F,
) -> Element<ContextMenuAtKind<F>>
where
    F: FnMut() -> Items + 'static,
    Items: ViewSeq,
{
    Element::new(ContextMenuAtKind { position, items })
}
