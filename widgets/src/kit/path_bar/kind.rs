use geometry::Size;
use layouting::Display;
use reactive::{Memo, Runtime, Signal, Store};
use semantics::Role;
use view::{Cx, Element, Kind, NodeId, StyleProp, Styled, keyed};

use super::{
    collapse::leading_to_fold,
    crumb::{Crumb, ELLIPSIS_WIDTH, ENTRY_GAP, Target, chevron, crumb},
    handlers::SegmentHandlers,
    segment::PathSegment,
};
use crate::{
    kit::{Theme, icons, use_theme},
    stack::hstack,
};

/// The kind behind [`path_bar`](super::path_bar).
pub struct PathBarKind {
    pub(super) segments: Signal<Vec<PathSegment>>,
    pub(super) handlers: SegmentHandlers,
}

impl Kind for PathBarKind {
    const NAME: &'static str = "kit_path_bar";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        cx.set_role(node, Role::Group);
        let handlers = cx.store(self.handlers);
        let drop_target = cx.signal(None);
        let available = cx.observe_size(node);
        let segments = self.segments;
        let path = keyed(
            move |runtime| segments.get(runtime),
            move |_, list: Vec<PathSegment>| {
                Element::new(PathRowKind {
                    list,
                    parts: Parts {
                        theme,
                        handlers,
                        drop_target,
                    },
                    available,
                })
                .flex_row()
                .items_center()
                .gap(ENTRY_GAP)
                .min_w(0.0)
            },
        );
        cx.set_children(node, path);
    }
}

#[derive(Clone, Copy)]
struct Parts {
    theme: Signal<Theme>,
    handlers: Store<SegmentHandlers>,
    drop_target: Signal<Option<usize>>,
}

/// One rendering of the path: an entry per segment behind the ellipsis that leading entries
/// fold into when the row runs out of room.
struct PathRowKind {
    list: Vec<PathSegment>,
    parts: Parts,
    available: Signal<Size>,
}

impl Kind for PathRowKind {
    const NAME: &'static str = "kit_path_row";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let nodes = self.parts.entries(cx, &self.list, self.available);
        cx.adopt_children(node, nodes);
    }
}

impl Parts {
    fn entries(self, cx: &mut Cx, list: &[PathSegment], available: Signal<Size>) -> Vec<NodeId> {
        let last = list.len().saturating_sub(1);
        let entries: Vec<NodeId> = list
            .iter()
            .enumerate()
            .map(|(index, segment)| {
                let pill = crumb(Crumb {
                    theme: self.theme,
                    segment,
                    target: Target::Segment(index),
                    current: index == last,
                    handlers: self.handlers,
                    drop_target: self.drop_target,
                });
                let separator = (index > 0).then(|| chevron(self.theme));
                cx.build(
                    hstack((separator, pill))
                        .items_center()
                        .gap(ENTRY_GAP)
                        .shrink(0.0),
                )
            })
            .collect();
        let folded = fold_to_fit(cx, &entries, available);
        let ellipsis = PathSegment::new("").icon(icons::ELLIPSIS);
        let more = cx.build(crumb(Crumb {
            theme: self.theme,
            segment: &ellipsis,
            target: Target::Folded(folded),
            current: false,
            handlers: self.handlers,
            drop_target: self.drop_target,
        }));
        show_while(cx, more, move |runtime| folded.get(runtime) > 0);
        for (index, entry) in entries.iter().enumerate() {
            show_while(cx, *entry, move |runtime| index >= folded.get(runtime));
        }
        let mut nodes = Vec::with_capacity(entries.len() + 1);
        nodes.push(more);
        nodes.extend(entries);
        nodes
    }
}

/// How many leading entries fold into the ellipsis; each entry's width is the last one it was
/// laid out at, so folded entries keep counting.
fn fold_to_fit(cx: &mut Cx, entries: &[NodeId], available: Signal<Size>) -> Memo<usize> {
    let sizes: Vec<Signal<Size>> = entries
        .iter()
        .map(|entry| cx.observe_size(*entry))
        .collect();
    let mut known = vec![0.0; sizes.len()];
    cx.memo(move |runtime| {
        for (width, size) in known.iter_mut().zip(&sizes) {
            let laid_out = size.get(runtime).width;
            if laid_out > 0.0 {
                *width = laid_out;
            }
        }
        let room = available.get(runtime).width;
        if room <= 0.0 || known.contains(&0.0) {
            return 0;
        }
        leading_to_fold(&known, room, ENTRY_GAP, ELLIPSIS_WIDTH)
    })
}

/// Takes `node` out of layout and of the semantic tree while `shown` is false.
fn show_while(cx: &mut Cx, node: NodeId, shown: impl Fn(&mut Runtime) -> bool + Copy + 'static) {
    cx.bind_style(node, move |runtime| {
        StyleProp::Display(if shown(runtime) {
            Display::Flex
        } else {
            Display::None
        })
    });
    cx.bind_style(node, move |runtime| StyleProp::Hidden(!shown(runtime)));
}
