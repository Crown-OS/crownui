use geometry::Vector;
use input::{Key, NamedKey};
use layouting::{Align, Dimension, Position};
use reactive::Signal;
use semantics::Role;
use view::{Cx, Kind, Listener, NodeId, SizeProperty, StyleProp, Styled, bind};
use windowing::CursorIcon;

use super::{
    item::ToggleItem,
    metrics::{HEIGHT, PILL_BORDER_WIDTH, SEGMENT_HEIGHT, TRACK_PADDING},
    segment::{SegmentState, segment},
    selection::Selection,
};
use crate::{
    kit::{INTER_FAMILY, use_theme},
    stack::vstack,
    util::{spring, themed::themed},
};

/// The kind behind [`toggle_group`](super::toggle_group).
pub struct ToggleGroupKind {
    pub(super) items: Vec<ToggleItem>,
    pub(super) selected: Signal<usize>,
    pub(super) stretch: bool,
}

impl Kind for ToggleGroupKind {
    const NAME: &'static str = "kit_toggle_group";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let count = self.items.len();
        let selection = Selection::new(self.selected, count);
        let disabled = cx.observe_disabled(node);
        let size = cx.observe_size(node);
        let hovered = cx.signal(None::<usize>);
        let segment_width = move |runtime: &mut reactive::Runtime| {
            (size.get(runtime).width - 2.0 * TRACK_PADDING) / count.max(1) as f32
        };

        cx.set_role(node, Role::Group);
        for prop in [
            StyleProp::GridColumns(vec![layouting::GridTrack::Fraction(1.0); count.max(1)]),
            StyleProp::Position(Position::Relative),
            StyleProp::AlignSelf(if self.stretch {
                Align::Stretch
            } else {
                Align::Start
            }),
            StyleProp::Size(SizeProperty::Height, Dimension::Length(HEIGHT)),
            StyleProp::Padding(view::Edges::All, TRACK_PADDING),
            StyleProp::Radius(view::Corners::All, HEIGHT / 2.0),
            StyleProp::Shrink(0.0),
            StyleProp::Cursor(CursorIcon::Pointer),
            StyleProp::FontFamily(INTER_FAMILY.to_owned()),
        ] {
            cx.set_default_style(node, prop);
        }
        cx.bind_style(node, move |runtime| {
            StyleProp::Background(Some(theme.with(runtime, |theme| theme.control.track)))
        });
        let labels: Vec<String> = self.items.iter().map(|item| item.label.clone()).collect();
        cx.bind_value(node, move |runtime| {
            labels
                .get(selection.get(runtime))
                .cloned()
                .unwrap_or_default()
        });

        let state = SegmentState {
            theme,
            disabled,
            selection,
            hovered,
        };
        let pill = vstack(())
            .w_frac(1.0 / count.max(1) as f32)
            .h(SEGMENT_HEIGHT)
            .rounded(SEGMENT_HEIGHT / 2.0)
            .bg(themed(theme, |theme| theme.popover.trigger_bg))
            .border(PILL_BORDER_WIDTH)
            .border_color(themed(theme, |theme| theme.popover.border))
            .translate(bind(move |runtime| {
                Vector::new(selection.get(runtime) as f32 * segment_width(runtime), 0.0)
            }))
            .transition(spring::snappy());
        let pill_node = cx.build(pill);

        let pill_lane = cx.build(vstack(()).absolute().inset(TRACK_PADDING));
        cx.adopt_children(pill_lane, vec![pill_node]);
        let mut children = vec![pill_lane];
        for (index, item) in self.items.iter().enumerate() {
            let segment = cx.build(segment(&state, index, item));
            cx.bind_checked(segment, move |runtime| selection.get(runtime) == index);
            children.push(segment);
        }
        cx.adopt_children(node, children);

        cx.add_listener(
            node,
            Listener::KeyDown(Box::new(move |cx, event| {
                let Key::Named(key) = event.key else {
                    return;
                };
                let target = match key {
                    NamedKey::ArrowLeft => selection.previous(cx),
                    NamedKey::ArrowRight => selection.next(cx),
                    NamedKey::Home => 0,
                    NamedKey::End => selection.last(),
                    _ => return,
                };
                event.prevent_default();
                if !disabled.get(cx) {
                    selection.choose(cx, target);
                }
            })),
        );
    }
}
