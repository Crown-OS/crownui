use std::{collections::HashMap, time::Duration};

use geometry::{Point, Vector};
use gesture::{
    ArenaDecision, ClickConfig, ClickRecognizer, DragConfig, DragRecognizer, GestureArena,
    GestureOutput, LongPressConfig, LongPressRecognizer, PinchRecognizer, Recognizer, Vote,
};
use input::{
    Key, KeyboardEvent, NamedKey, PointerButton, PointerEvent, PointerId, TextInputEvent,
    TouchpadGesture, WheelEvent,
};
use layouting::Overflow;
use smallvec::SmallVec;

use crate::{
    NodeId,
    event::{ClickEvent, DragEvent, Event, FocusEvent, LongPressEvent, PinchEvent, ScrollEvent},
    handler::{Dispatchable, ListenerKind},
    style_prop::StyleProp,
    ui::Ui,
};

const WHEEL_LINE_HEIGHT: f32 = 40.0;

/// Which recognizer of a node an arena member is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum RecognizerSlot {
    Click,
    ContextClick,
    LongPress,
    Drag,
    Pinch,
}

pub(crate) type GestureMember = (NodeId, RecognizerSlot);

/// Recognizers of one node; they persist across presses so click counts accumulate.
#[derive(Default)]
pub(crate) struct NodeRecognizers {
    click: Option<ClickRecognizer>,
    context_click: Option<ClickRecognizer>,
    long_press: Option<LongPressRecognizer>,
    drag: Option<DragRecognizer>,
    pinch: Option<PinchRecognizer>,
}

impl NodeRecognizers {
    fn get(&mut self, slot: RecognizerSlot) -> Option<&mut dyn Recognizer> {
        match slot {
            RecognizerSlot::Click => self
                .click
                .as_mut()
                .map(|recognizer| recognizer as &mut dyn Recognizer),
            RecognizerSlot::ContextClick => self
                .context_click
                .as_mut()
                .map(|recognizer| recognizer as &mut dyn Recognizer),
            RecognizerSlot::LongPress => self
                .long_press
                .as_mut()
                .map(|recognizer| recognizer as &mut dyn Recognizer),
            RecognizerSlot::Drag => self
                .drag
                .as_mut()
                .map(|recognizer| recognizer as &mut dyn Recognizer),
            RecognizerSlot::Pinch => self
                .pinch
                .as_mut()
                .map(|recognizer| recognizer as &mut dyn Recognizer),
        }
    }
}

/// Per-pointer dispatch state.
#[derive(Default)]
pub(crate) struct PointerTrack {
    members: SmallVec<[GestureMember; 4]>,
}

/// Input routing state owned by [`Ui`].
#[derive(Default)]
pub(crate) struct Dispatcher {
    pub(crate) recognizers: HashMap<NodeId, NodeRecognizers>,
    pub(crate) arena: GestureArena<GestureMember>,
    pub(crate) pointers: HashMap<PointerId, PointerTrack>,
    pub(crate) hovered: Vec<NodeId>,
    pub(crate) pressed: Option<NodeId>,
    pub(crate) last_pointer: Point,
}

impl Ui {
    fn ensure_recognizers(&mut self, node: NodeId) {
        let Some(element) = self.cx.tree.get(node) else {
            return;
        };
        let wants = |kind: ListenerKind| element.listens_to(kind);
        let click = wants(ListenerKind::Click) || wants(ListenerKind::DoubleClick);
        let context = wants(ListenerKind::ContextMenu);
        let long_press = wants(ListenerKind::LongPress);
        let drag = wants(ListenerKind::DragStart)
            || wants(ListenerKind::Drag)
            || wants(ListenerKind::DragEnd);
        let pinch = wants(ListenerKind::Pinch);
        if !(click || context || long_press || drag || pinch) {
            return;
        }
        let entry = self.dispatcher.recognizers.entry(node).or_default();
        if click && entry.click.is_none() {
            entry.click = Some(ClickRecognizer::new(ClickConfig::default()));
        }
        if context && entry.context_click.is_none() {
            entry.context_click = Some(ClickRecognizer::new(ClickConfig {
                button: PointerButton::Secondary,
                ..ClickConfig::default()
            }));
        }
        if long_press && entry.long_press.is_none() {
            entry.long_press = Some(LongPressRecognizer::new(LongPressConfig::default()));
        }
        if drag && entry.drag.is_none() {
            entry.drag = Some(DragRecognizer::new(DragConfig::default()));
        }
        if pinch && entry.pinch.is_none() {
            entry.pinch = Some(PinchRecognizer::default());
        }
    }

    fn with_recognizer<R>(
        &mut self,
        member: GestureMember,
        act: impl FnOnce(&mut dyn Recognizer) -> R,
    ) -> Option<R> {
        self.dispatcher
            .recognizers
            .get_mut(&member.0)
            .and_then(|recognizers| recognizers.get(member.1))
            .map(act)
    }

    fn apply_decisions(
        &mut self,
        decisions: impl IntoIterator<Item = ArenaDecision<GestureMember>>,
    ) {
        for decision in decisions {
            match decision {
                ArenaDecision::Won(member) => {
                    let mut outputs = Vec::new();
                    self.with_recognizer(member, |recognizer| {
                        recognizer.accepted();
                        recognizer.drain_outputs(&mut outputs);
                    });
                    self.deliver_outputs(member, outputs);
                }
                ArenaDecision::Lost(member) => {
                    self.with_recognizer(member, |recognizer| recognizer.rejected());
                }
            }
        }
    }

    fn vote(&mut self, pointer: PointerId, member: GestureMember, vote: Vote) {
        if vote == Vote::Pending {
            return;
        }
        let decisions = self.dispatcher.arena.resolve(pointer, member, vote);
        self.apply_decisions(decisions);
    }

    fn drain_winners(&mut self, pointer: PointerId) {
        let members: SmallVec<[GestureMember; 4]> = self
            .dispatcher
            .pointers
            .get(&pointer)
            .map(|track| track.members.clone())
            .unwrap_or_default();
        for member in members {
            let mut outputs = Vec::new();
            self.with_recognizer(member, |recognizer| recognizer.drain_outputs(&mut outputs));
            if !outputs.is_empty() {
                self.deliver_outputs(member, outputs);
            }
        }
    }

    fn deliver_outputs(&mut self, (node, slot): GestureMember, outputs: Vec<GestureOutput>) {
        let origin = self
            .cx
            .tree
            .get(node)
            .map(|element| element.geometry.window_rect.origin)
            .unwrap_or(Point::ZERO);
        let local = |position: Point| Point::new(position.x - origin.x, position.y - origin.y);
        for output in outputs {
            let mut dispatchable = match output {
                GestureOutput::Click { position, count } => {
                    let click = ClickEvent {
                        position: local(position),
                        count: u32::from(count),
                    };
                    if slot == RecognizerSlot::ContextClick {
                        Dispatchable::Click(ListenerKind::ContextMenu, Event::new(click, false))
                    } else {
                        if count == 2 {
                            let mut double = Dispatchable::Click(
                                ListenerKind::DoubleClick,
                                Event::new(click, false),
                            );
                            self.bubble(node, &mut double);
                        }
                        Dispatchable::Click(ListenerKind::Click, Event::new(click, false))
                    }
                }
                GestureOutput::LongPress { position } => Dispatchable::LongPress(Event::new(
                    LongPressEvent {
                        position: local(position),
                    },
                    false,
                )),
                GestureOutput::DragStart { position } => Dispatchable::Drag(
                    ListenerKind::DragStart,
                    Event::new(
                        DragEvent {
                            position: local(position),
                            delta: Vector::ZERO,
                            total: Vector::ZERO,
                            velocity: Vector::ZERO,
                        },
                        false,
                    ),
                ),
                GestureOutput::Drag {
                    position,
                    delta,
                    total,
                } => Dispatchable::Drag(
                    ListenerKind::Drag,
                    Event::new(
                        DragEvent {
                            position: local(position),
                            delta,
                            total,
                            velocity: Vector::ZERO,
                        },
                        false,
                    ),
                ),
                GestureOutput::DragEnd { position, velocity } => Dispatchable::Drag(
                    ListenerKind::DragEnd,
                    Event::new(
                        DragEvent {
                            position: local(position),
                            delta: Vector::ZERO,
                            total: Vector::ZERO,
                            velocity,
                        },
                        false,
                    ),
                ),
                GestureOutput::Pinch {
                    scale, rotation, ..
                } => Dispatchable::Pinch(Event::new(PinchEvent { scale, rotation }, false)),
                GestureOutput::PinchStart
                | GestureOutput::PinchEnd { .. }
                | GestureOutput::Cancel => continue,
            };
            if self
                .cx
                .tree
                .get(node)
                .is_some_and(|element| element.disabled)
            {
                continue;
            }
            self.bubble(node, &mut dispatchable);
        }
    }

    /// Delivers `event` to `target` and its ancestors until a handler stops propagation.
    pub(crate) fn bubble(&mut self, target: NodeId, event: &mut Dispatchable) {
        let path: Vec<NodeId> = self.cx.tree.ancestors(target).collect();
        self.dispatch_along(&path, event);
    }

    pub(crate) fn dispatch_along(&mut self, path: &[NodeId], event: &mut Dispatchable) {
        let kind = event.kind();
        for node in path {
            let has_listener = self
                .cx
                .tree
                .get(*node)
                .is_some_and(|element| element.listens_to(kind));
            if has_listener {
                let mut listeners = self
                    .cx
                    .tree
                    .get_mut(*node)
                    .map(|element| std::mem::take(&mut element.listeners))
                    .unwrap_or_default();
                for listener in listeners
                    .iter_mut()
                    .filter(|listener| listener.kind() == kind)
                {
                    event.invoke(listener, &mut self.cx);
                }
                if let Some(element) = self.cx.tree.get_mut(*node) {
                    listeners.append(&mut element.listeners);
                    element.listeners = listeners;
                }
            }
            if event.is_propagation_stopped() {
                break;
            }
        }
        self.cx.apply_pending();
        self.apply_focus_request();
        for (node, mut deferred) in std::mem::take(&mut self.cx.deferred) {
            self.bubble(node, &mut deferred);
        }
    }

    fn set_interaction(
        &mut self,
        node: NodeId,
        change: impl FnOnce(&mut crate::node::InteractionState),
    ) {
        let Some(element) = self.cx.tree.get_mut(node) else {
            return;
        };
        let before = element.interaction;
        change(&mut element.interaction);
        let has_variants = !element.variants.is_empty();
        if before != element.interaction && has_variants {
            self.cx.refresh_visual(node);
        }
    }

    pub(crate) fn pointer_leave(&mut self, pointer: &PointerEvent) {
        for node in std::mem::take(&mut self.dispatcher.hovered) {
            self.set_interaction(node, |state| state.hovered = false);
            let mut event =
                Dispatchable::Pointer(ListenerKind::PointerLeave, Event::new(*pointer, false));
            self.dispatch_along(&[node], &mut event);
        }
    }

    pub(crate) fn update_hover(&mut self, position: Point, pointer: &PointerEvent) {
        let path = self.cx.tree.hit_test(position);
        let previous = std::mem::take(&mut self.dispatcher.hovered);
        for node in previous.iter().filter(|node| !path.contains(node)) {
            self.set_interaction(*node, |state| state.hovered = false);
            let mut event =
                Dispatchable::Pointer(ListenerKind::PointerLeave, Event::new(*pointer, false));
            self.dispatch_along(&[*node], &mut event);
        }
        for node in path.iter().filter(|node| !previous.contains(node)) {
            self.set_interaction(*node, |state| state.hovered = true);
            let mut event =
                Dispatchable::Pointer(ListenerKind::PointerEnter, Event::new(*pointer, false));
            self.dispatch_along(&[*node], &mut event);
        }
        self.dispatcher.hovered = path;
    }

    pub(crate) fn pointer_down(&mut self, event: &PointerEvent) {
        self.dispatcher.last_pointer = event.position;
        let path = self.cx.tree.hit_test(event.position);
        let pointer = event.pointer_id;
        let mut members: SmallVec<[GestureMember; 4]> = SmallVec::new();
        let mut votes = Vec::new();
        for node in &path {
            if self
                .cx
                .tree
                .get(*node)
                .is_some_and(|element| element.disabled)
            {
                continue;
            }
            self.ensure_recognizers(*node);
            for slot in [
                RecognizerSlot::Click,
                RecognizerSlot::ContextClick,
                RecognizerSlot::LongPress,
                RecognizerSlot::Drag,
            ] {
                let member = (*node, slot);
                let joined = self.with_recognizer(member, |recognizer| {
                    recognizer
                        .allows_pointer(event)
                        .then(|| recognizer.pointer_down(event))
                });
                if let Some(Some(vote)) = joined {
                    self.dispatcher.arena.add(pointer, member);
                    members.push(member);
                    votes.push((member, vote));
                }
            }
        }
        let closed = self.dispatcher.arena.close(pointer);
        self.dispatcher
            .pointers
            .insert(pointer, PointerTrack { members });
        for (member, vote) in votes {
            self.vote(pointer, member, vote);
        }
        self.apply_decisions(closed);
        let pressed = path.iter().copied().find(|node| {
            self.cx
                .tree
                .get(*node)
                .is_some_and(|element| element.listens_to(ListenerKind::Click) && !element.disabled)
        });
        if let Some(node) = pressed {
            self.set_interaction(node, |state| state.pressed = true);
            self.dispatcher.pressed = Some(node);
        }
        let mut dom_event =
            Dispatchable::Pointer(ListenerKind::PointerDown, Event::new(*event, false));
        self.dispatch_along(&path, &mut dom_event);
        if !dom_event.is_default_prevented() {
            let target = path.iter().copied().find(|node| {
                self.cx
                    .tree
                    .get(*node)
                    .is_some_and(crate::node::ElementNode::is_focusable)
            });
            match target {
                Some(node) => self.set_focus(Some(node), false),
                None => self.set_focus(None, false),
            }
        }
    }

    pub(crate) fn pointer_move(&mut self, event: &PointerEvent) {
        self.dispatcher.last_pointer = event.position;
        self.update_hover(event.position, event);
        let pointer = event.pointer_id;
        let members = self
            .dispatcher
            .pointers
            .get(&pointer)
            .map(|track| track.members.clone())
            .unwrap_or_default();
        for member in members {
            if let Some(vote) =
                self.with_recognizer(member, |recognizer| recognizer.pointer_move(event))
            {
                self.vote(pointer, member, vote);
            }
        }
        self.drain_winners(pointer);
        let path = self.cx.tree.hit_test(event.position);
        let mut dom_event =
            Dispatchable::Pointer(ListenerKind::PointerMove, Event::new(*event, false));
        self.dispatch_along(&path, &mut dom_event);
    }

    pub(crate) fn pointer_up(&mut self, event: &PointerEvent) {
        let pointer = event.pointer_id;
        let members = self
            .dispatcher
            .pointers
            .remove(&pointer)
            .map(|track| track.members)
            .unwrap_or_default();
        self.dispatcher.pointers.insert(
            pointer,
            PointerTrack {
                members: members.clone(),
            },
        );
        for member in &members {
            if let Some(vote) =
                self.with_recognizer(*member, |recognizer| recognizer.pointer_up(event))
            {
                self.vote(pointer, *member, vote);
            }
        }
        let swept = self.dispatcher.arena.sweep(pointer);
        self.apply_decisions(swept);
        self.drain_winners(pointer);
        let released = self.dispatcher.arena.release(pointer);
        self.apply_decisions(released);
        self.dispatcher.pointers.remove(&pointer);
        if let Some(node) = self.dispatcher.pressed.take() {
            self.set_interaction(node, |state| state.pressed = false);
        }
        let path = self.cx.tree.hit_test(event.position);
        let mut dom_event =
            Dispatchable::Pointer(ListenerKind::PointerUp, Event::new(*event, false));
        self.dispatch_along(&path, &mut dom_event);
    }

    pub(crate) fn pointer_cancel(&mut self, event: &PointerEvent) {
        let pointer = event.pointer_id;
        if let Some(track) = self.dispatcher.pointers.remove(&pointer) {
            for member in track.members {
                self.with_recognizer(member, |recognizer| recognizer.pointer_cancel());
            }
        }
        let released = self.dispatcher.arena.release(pointer);
        self.apply_decisions(released);
        if let Some(node) = self.dispatcher.pressed.take() {
            self.set_interaction(node, |state| state.pressed = false);
        }
        let path = self.cx.tree.hit_test(event.position);
        let mut dom_event =
            Dispatchable::Pointer(ListenerKind::PointerCancel, Event::new(*event, false));
        self.dispatch_along(&path, &mut dom_event);
    }

    pub(crate) fn poll_recognizers(&mut self, now: Duration) {
        let active: Vec<(PointerId, GestureMember)> = self
            .dispatcher
            .pointers
            .iter()
            .flat_map(|(pointer, track)| {
                track.members.iter().map(move |member| (*pointer, *member))
            })
            .collect();
        for (pointer, member) in active {
            if let Some(vote) = self.with_recognizer(member, |recognizer| recognizer.poll(now)) {
                self.vote(pointer, member, vote);
            }
        }
    }

    pub(crate) fn touchpad(&mut self, gesture: &TouchpadGesture) {
        let path = self.cx.tree.hit_test(self.dispatcher.last_pointer);
        for node in path {
            self.ensure_recognizers(node);
            let member = (node, RecognizerSlot::Pinch);
            if let Some(vote) =
                self.with_recognizer(member, |recognizer| recognizer.touchpad(gesture))
                && vote == Vote::Accept
            {
                let mut outputs = Vec::new();
                self.with_recognizer(member, |recognizer| {
                    recognizer.accepted();
                    recognizer.drain_outputs(&mut outputs);
                });
                self.deliver_outputs(member, outputs);
                return;
            }
        }
    }

    pub(crate) fn wheel(&mut self, event: &WheelEvent, synthetic: bool) {
        let path = self.cx.tree.hit_test(event.position);
        let mut dom_event = Dispatchable::Wheel(Event::new(*event, synthetic));
        self.dispatch_along(&path, &mut dom_event);
        if dom_event.is_default_prevented() {
            return;
        }
        let delta = event.delta.to_pixels(WHEEL_LINE_HEIGHT);
        self.scroll_nearest(&path, delta, synthetic);
    }

    pub(crate) fn scroll_nearest(&mut self, path: &[NodeId], delta: Vector, synthetic: bool) {
        let container = path.iter().copied().find(|node| {
            self.cx
                .tree
                .get(*node)
                .is_some_and(|element| element.style.layout.overflow == Overflow::Scroll)
        });
        let Some(node) = container else {
            return;
        };
        let Some(element) = self.cx.tree.get(node) else {
            return;
        };
        let geometry = element.geometry;
        let max = Vector::new(
            (geometry.content_size.width - geometry.rect.size.width).max(0.0),
            (geometry.content_size.height - geometry.rect.size.height).max(0.0),
        );
        let current = element.style.visual.content_offset;
        let next = Vector::new(
            (current.x + delta.x).clamp(0.0, max.x),
            (current.y + delta.y).clamp(0.0, max.y),
        );
        if next == current {
            return;
        }
        self.cx.apply_style(node, &StyleProp::ContentOffset(next));
        if let Some(element) = self.cx.tree.get_mut(node) {
            element.dirty.frame = true;
            for child in element.children.clone() {
                if let Some(child) = self.cx.tree.get_mut(child) {
                    child.dirty.frame = true;
                }
            }
        }
        self.layout_needed = true;
        let mut scroll = Dispatchable::Scroll(Event::new(ScrollEvent { offset: next }, synthetic));
        self.bubble(node, &mut scroll);
    }

    pub(crate) fn key_down(&mut self, event: &KeyboardEvent, synthetic: bool) {
        let shortcut_target = self
            .cx
            .tree
            .iter()
            .find(|(_, element)| {
                element
                    .shortcut
                    .is_some_and(|keybind| keybind.matches(event))
                    && !element.disabled
            })
            .map(|(id, _)| id);
        if let Some(node) = shortcut_target {
            self.click(node, synthetic);
            return;
        }
        let path: Vec<NodeId> = match self.focused {
            Some(node) => self.cx.tree.ancestors(node).collect(),
            None => self.cx.tree.root().into_iter().collect(),
        };
        let mut dom_event =
            Dispatchable::Key(ListenerKind::KeyDown, Event::new(event.clone(), synthetic));
        self.dispatch_along(&path, &mut dom_event);
        if dom_event.is_default_prevented() {
            return;
        }
        let activates = matches!(event.key, Key::Named(NamedKey::Enter | NamedKey::Space))
            && event.modifiers.is_empty();
        if activates
            && let Some(node) = self.focused
            && self
                .cx
                .tree
                .get(node)
                .is_some_and(|element| element.listens_to(ListenerKind::Click))
        {
            self.click(node, synthetic);
            return;
        }
        if let Some(change) = self.focus_manager.handle_key(event) {
            let target = self.focus_manager.focused();
            let _ = change;
            self.set_focus(target, true);
        }
    }

    pub(crate) fn key_up(&mut self, event: &KeyboardEvent, synthetic: bool) {
        let path: Vec<NodeId> = match self.focused {
            Some(node) => self.cx.tree.ancestors(node).collect(),
            None => self.cx.tree.root().into_iter().collect(),
        };
        let mut dom_event =
            Dispatchable::Key(ListenerKind::KeyUp, Event::new(event.clone(), synthetic));
        self.dispatch_along(&path, &mut dom_event);
    }

    pub(crate) fn text_input(&mut self, event: &TextInputEvent, synthetic: bool) {
        let Some(node) = self.focused else {
            return;
        };
        let mut dom_event = Dispatchable::TextInput(Event::new(event.clone(), synthetic));
        self.bubble(node, &mut dom_event);
    }

    /// Clicks `node` as a keyboard activation, an agent or assistive technology would.
    pub(crate) fn click(&mut self, node: NodeId, synthetic: bool) {
        if self
            .cx
            .tree
            .get(node)
            .is_none_or(|element| element.disabled)
        {
            return;
        }
        let center = self
            .cx
            .tree
            .get(node)
            .map(|element| {
                let size = element.geometry.rect.size;
                Point::new(size.width / 2.0, size.height / 2.0)
            })
            .unwrap_or(Point::ZERO);
        let mut event = Dispatchable::Click(
            ListenerKind::Click,
            Event::new(
                ClickEvent {
                    position: center,
                    count: 1,
                },
                synthetic,
            ),
        );
        self.bubble(node, &mut event);
    }

    pub(crate) fn set_focus(&mut self, target: Option<NodeId>, by_keyboard: bool) {
        if target == self.focused {
            if let Some(node) = target {
                self.set_interaction(node, |state| state.focus_visible = by_keyboard);
            }
            return;
        }
        if let Some(previous) = self.focused.take() {
            self.set_interaction(previous, |state| {
                state.focused = false;
                state.focus_visible = false;
            });
            let mut blur = Dispatchable::Focus(
                ListenerKind::Blur,
                Event::new(
                    FocusEvent {
                        visible: by_keyboard,
                    },
                    false,
                ),
            );
            self.dispatch_along(&[previous], &mut blur);
        }
        self.focused = target;
        match target {
            Some(node) => {
                self.focus_manager.focus(node);
                self.set_interaction(node, |state| {
                    state.focused = true;
                    state.focus_visible = by_keyboard;
                });
                let mut focus = Dispatchable::Focus(
                    ListenerKind::Focus,
                    Event::new(
                        FocusEvent {
                            visible: by_keyboard,
                        },
                        false,
                    ),
                );
                self.dispatch_along(&[node], &mut focus);
            }
            None => {
                self.focus_manager.blur();
            }
        }
        self.semantics_dirty = true;
    }

    fn apply_focus_request(&mut self) {
        if let Some(node) = self.cx.focus_request.take() {
            self.set_focus(Some(node), true);
        }
    }

    pub(crate) fn forget_nodes(&mut self, removed: &[NodeId]) {
        self.forget_drop_nodes(removed);
        for node in removed {
            self.dispatcher.recognizers.remove(node);
            self.dispatcher.hovered.retain(|hovered| hovered != node);
            if self.dispatcher.pressed == Some(*node) {
                self.dispatcher.pressed = None;
            }
            if self.focused == Some(*node) {
                self.focused = None;
                let _ = self.focus_manager.blur();
            }
        }
    }
}
