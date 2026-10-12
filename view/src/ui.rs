use std::{any::Any, collections::HashMap, time::Duration};

use geometry::{Rect, Size};
use gesture::FocusManager;
use graphics::TextSystem;
use input::{Keybind, Modifiers};
use layouting::LayoutEngine;
use semantics::{SemanticDiff, SemanticId, SemanticTree, UiAction};
use windowing::{CursorIcon, Material, MaterialShape, SurfaceState, WindowEvent};

use crate::{
    Cx, NodeId, View,
    color_scheme::ColorScheme,
    dispatch::Dispatcher,
    drop_target::DropHover,
    layout::LayoutState,
    render_op::{NodeKey, RenderOp, UiFrame},
    semantics_build,
    tasks::{TaskExecutor, TaskId},
    text_cache::TextCache,
};

/// Why an agent or test action could not be performed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ActionError {
    /// The id names no node in the current tree.
    #[error("CUI0103: no node with semantic id {0}")]
    UnknownNode(u64),
    /// The node does not support the action.
    #[error("CUI0104: {action} is not supported by {role} {path}")]
    Unsupported {
        /// Action name.
        action: &'static str,
        /// Node role.
        role: String,
        /// Component path.
        path: String,
    },
    /// The node holds a secret.
    #[error("CUI0105: {0} is secure; agents may not type into it")]
    Secure(String),
    /// The node is disabled.
    #[error("CUI0106: {0} is disabled")]
    Disabled(String),
}

/// The UI-thread half of a CrownUI app: builds the view, routes input, runs reactive updates,
/// lays out, and produces render ops and the semantic tree. Generic code (layout engines, text
/// systems) is passed in per call, so the UI itself stays non-generic.
pub struct Ui {
    pub(crate) cx: Cx,
    pub(crate) dispatcher: Dispatcher,
    pub(crate) drop_hover: DropHover,
    pub(crate) focus_manager: FocusManager<NodeId>,
    pub(crate) focused: Option<NodeId>,
    pub(crate) semantics_dirty: bool,
    pub(crate) layout_needed: bool,
    viewport: Size,
    semantics: SemanticTree,
    semantic_index: HashMap<SemanticId, NodeId>,
    pending_diff: Option<SemanticDiff>,
    removed_for_render: Vec<(NodeKey, Option<(crate::style::Visual, motion::Transaction)>)>,
    sent_cursor: Option<CursorIcon>,
    sent_material: Option<(Option<Material>, Vec<MaterialShape>)>,
    sent_input_region: Option<Vec<Rect>>,
    emulate_materials: bool,
    rtl: bool,
    now: Duration,
}

impl Ui {
    /// Builds `root` into a fresh tree.
    pub fn new(root: impl View) -> Self {
        let mut cx = Cx::new();
        let node = root.build(&mut cx);
        cx.tree.set_root(node);
        cx.fill_viewport(node);
        cx.apply_pending();
        cx.assign_semantic_ids();
        Self {
            cx,
            dispatcher: Dispatcher::default(),
            drop_hover: DropHover::default(),
            focus_manager: FocusManager::default(),
            focused: None,
            semantics_dirty: true,
            layout_needed: true,
            viewport: Size::ZERO,
            semantics: SemanticTree::new(),
            semantic_index: HashMap::new(),
            pending_diff: None,
            removed_for_render: Vec::new(),
            sent_cursor: None,
            sent_material: None,
            sent_input_region: None,
            emulate_materials: false,
            rtl: false,
            now: Duration::ZERO,
        }
    }

    /// The context, e.g. to change signals from outside the tree.
    pub fn cx(&mut self) -> &mut Cx {
        &mut self.cx
    }

    /// Installs the executor that runs `cx.spawn` futures.
    pub fn set_task_executor(&mut self, executor: Box<dyn TaskExecutor>) {
        self.cx.tasks.set_executor(executor);
    }

    /// Delivers a finished task's output.
    pub fn complete_task(&mut self, id: TaskId, output: Box<dyn Any + Send>) {
        self.cx.complete_task(id, output);
        self.cx.apply_pending();
    }

    /// Draws `.material(..)` nodes with in-app backdrop blur when the platform cannot.
    pub fn set_emulate_materials(&mut self, emulate: bool) {
        if self.emulate_materials != emulate {
            self.emulate_materials = emulate;
            for (_, element) in self.cx.tree.iter_all_mut() {
                if element.style.material.is_some() {
                    element.dirty.visual = true;
                }
            }
        }
    }

    /// Lays everything out right to left (mirrored rows, RTL text) when `rtl` is true.
    pub fn set_right_to_left(&mut self, rtl: bool) {
        if self.rtl == rtl {
            return;
        }
        self.rtl = rtl;
        for (_, element) in self.cx.tree.iter_all_mut() {
            element.dirty.style = true;
            element.dirty.measure = true;
            element.dirty.frame = true;
        }
        self.layout_needed = true;
    }

    /// Switches light/dark appearance.
    pub fn set_color_scheme(&mut self, scheme: ColorScheme) {
        let signal = self.cx.color_scheme;
        signal.set(&mut self.cx, scheme);
        self.cx.apply_pending();
    }

    /// The window's content size in logical pixels.
    pub fn viewport(&self) -> Size {
        self.viewport
    }

    /// Sets the window's content size.
    pub fn set_viewport(&mut self, size: Size) {
        if size != self.viewport {
            self.viewport = size;
            self.layout_needed = true;
        }
    }

    /// Routes a window event into the tree.
    pub fn handle_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::PointerDown(pointer) => self.pointer_down(pointer),
            WindowEvent::PointerUp(pointer) => self.pointer_up(pointer),
            WindowEvent::PointerMove(pointer) | WindowEvent::PointerEnter(pointer) => {
                self.pointer_move(pointer)
            }
            WindowEvent::PointerLeave(pointer) => self.pointer_leave(pointer),
            WindowEvent::PointerCancel(pointer) => self.pointer_cancel(pointer),
            WindowEvent::Wheel(wheel) => self.wheel(wheel, false),
            WindowEvent::KeyDown(key) => self.key_down(key, false),
            WindowEvent::KeyUp(key) => self.key_up(key, false),
            WindowEvent::TextInput(text) => self.text_input(text, false),
            WindowEvent::ClipboardText(pasted) => {
                self.text_input(&input::TextInputEvent::Commit(pasted.clone()), false)
            }
            WindowEvent::TouchpadGesture(gesture) => self.touchpad(gesture),
            WindowEvent::DragEnter(offer) => self.drag_enter(offer),
            WindowEvent::DragOver(position) => self.drag_over(*position),
            WindowEvent::DragLeave => self.drag_leave(),
            WindowEvent::Drop(dropped) => self.drop_data(dropped),
            WindowEvent::DragEnded(outcome) => {
                if let Some(on_end) = self.cx.drag_end.take() {
                    on_end(&mut self.cx, *outcome);
                    self.cx.apply_pending();
                }
            }
            WindowEvent::LaunchApp(app_id) => self.cx.launch_app(app_id),
            WindowEvent::Focused(false) => self.set_focus(None, false),
            WindowEvent::Visibility { .. }
            | WindowEvent::Resized(_)
            | WindowEvent::ScaleFactorChanged(_)
            | WindowEvent::CloseRequested
            | WindowEvent::RedrawRequested
            | WindowEvent::Focused(true)
            | WindowEvent::ModifiersChanged(_) => {}
        }
        self.semantics_dirty = true;
    }

    /// Advances time-based work (long presses, task cancellation) and applies pending updates.
    pub fn update(&mut self, now: Duration) {
        self.now = now;
        self.poll_recognizers(now);
        let mut tasks = std::mem::take(&mut self.cx.tasks);
        tasks.cancel_dead(|scope| self.cx.is_scope_alive(scope));
        self.cx.tasks = tasks;
        self.cx.apply_pending();
    }

    /// Tells components what the platform supports (see [`Cx::platform_capabilities`]).
    pub fn set_platform_capabilities(&mut self, capabilities: windowing::Capabilities) {
        let signal = self.cx.capabilities;
        signal.set(&mut self.cx, capabilities);
        self.cx.apply_pending();
    }

    /// Sets the window's scale factor (see [`Cx::scale_factor`]).
    pub fn set_scale_factor(&mut self, scale: f32) {
        let signal = self.cx.scale_factor;
        if signal.with_untracked(&self.cx, |current| *current != scale) {
            signal.set(&mut self.cx, scale);
            self.cx.apply_pending();
        }
    }

    /// The drag a handler started with [`Cx::start_drag`], for the platform to begin. Pointers
    /// held for it are cancelled: the platform owns the pointer until the drag ends.
    pub fn take_drag_request(&mut self) -> Option<windowing::DragSource> {
        let source = self.cx.drag_request.take()?;
        let held: Vec<input::PointerId> = self.dispatcher.pointers.keys().copied().collect();
        for pointer_id in held {
            let mut pointer = input::PointerEvent::mouse(self.dispatcher.last_pointer);
            pointer.pointer_id = pointer_id;
            self.pointer_cancel(&pointer);
        }
        Some(source)
    }

    /// App icon changes since the last call (see [`Cx::set_app_icon`]), for the platform.
    pub fn take_app_icon_requests(&mut self) -> Vec<crate::AppIconRequest> {
        self.cx.app_icons.take_requests()
    }

    /// Whether a component asked for an animation frame (see [`Cx::request_animation_frame`]).
    pub fn wants_animation_frame(&self) -> bool {
        !self.cx.animation_frames.is_empty()
    }

    /// Runs the queued animation-frame callbacks for the frame at `now`.
    pub fn run_animation_frames(&mut self, now: Duration) {
        self.cx.run_animation_frames(now);
    }

    /// Whether the app asked to keep the system from idling (see [`Cx::set_idle_inhibited`]).
    pub fn idle_inhibit_request(&self) -> bool {
        self.cx.idle_inhibited
    }

    /// Whether anything waits to be laid out or rendered.
    pub fn needs_frame(&self) -> bool {
        self.layout_needed
            || self.cx.has_pending()
            || self.cx.tree.iter().any(|(_, element)| {
                let dirty = element.dirty;
                dirty.style
                    || dirty.children
                    || dirty.measure
                    || dirty.visual
                    || dirty.content
                    || dirty.frame
            })
    }

    /// Lays out the tree with `layout` and `text`.
    pub fn layout<L: LayoutEngine, T: TextSystem>(
        &mut self,
        layout: &mut LayoutState<L>,
        text: &mut TextCache<T>,
    ) {
        let removed = self.cx.tree.take_removed();
        let removed_ids: Vec<NodeId> = removed.iter().map(|gone| gone.id).collect();
        self.forget_nodes(&removed_ids);
        self.removed_for_render.extend(
            removed
                .iter()
                .filter(|gone| gone.rendered && !gone.inside_exit)
                .map(|gone| {
                    let exit = gone.exit.map(|(visual, curve)| {
                        (
                            visual,
                            motion::Transaction::new(
                                curve.unwrap_or(motion::Curve::Spring(motion::Spring::smooth())),
                            ),
                        )
                    });
                    (NodeKey::of(gone.id), exit)
                }),
        );
        let outcome = self
            .cx
            .tree
            .layout(layout, text, self.viewport, &removed, self.rtl);
        for (signal, size) in outcome.canvas_sizes {
            if signal.with_untracked(&self.cx, |current| *current != size) {
                signal.set(&mut self.cx, size);
            }
        }
        self.cx.apply_pending();
        if outcome.frames_changed || !removed.is_empty() {
            self.semantics_dirty = true;
        }
        self.layout_needed = false;
        if self
            .cx
            .tree
            .iter()
            .any(|(_, element)| element.dirty.style || element.dirty.children)
        {
            self.layout_needed = true;
        }
    }

    /// Takes every render change since the last frame.
    pub fn take_frame(&mut self) -> UiFrame {
        let mut ops: Vec<RenderOp> = self
            .removed_for_render
            .drain(..)
            .map(|(key, exit)| RenderOp::Remove(key, exit))
            .collect();
        let explicit = self.cx.frame_transaction.take().or(self.cx.transaction);
        self.cx.assign_semantic_ids();
        self.cx
            .tree
            .collect_render_ops(explicit, self.emulate_materials, &mut ops);
        if !ops.is_empty() {
            self.semantics_dirty = true;
        }
        let (material, shapes) = self.cx.tree.material_shapes();
        let mut surface =
            (self.sent_material.as_ref() != Some(&(material, shapes.clone()))).then(|| {
                self.sent_material = Some((material, shapes.clone()));
                SurfaceState {
                    material: Some(material.unwrap_or_default()),
                    material_shapes: Some(shapes),
                    ..SurfaceState::default()
                }
            });
        let input_region = self.cx.tree.input_region();
        if input_region != self.sent_input_region {
            let everywhere = Rect::new(
                geometry::Point::ZERO,
                geometry::Size::new(f32::from(i16::MAX), f32::from(i16::MAX)),
            );
            surface
                .get_or_insert_with(SurfaceState::default)
                .input_region = Some(input_region.clone().unwrap_or_else(|| vec![everywhere]));
            self.sent_input_region = input_region;
        }
        let cursor = self
            .dispatcher
            .hovered
            .first()
            .and_then(|node| {
                self.cx.tree.ancestors(*node).find_map(|ancestor| {
                    self.cx
                        .tree
                        .get(ancestor)
                        .and_then(|element| element.style.cursor)
                })
            })
            .unwrap_or_default();
        let cursor = (self.sent_cursor != Some(cursor)).then(|| {
            self.sent_cursor = Some(cursor);
            cursor
        });
        if self.semantics_dirty {
            self.refresh_semantics();
        }
        let images = std::mem::take(&mut self.cx.image_uploads);
        let image_releases = std::mem::take(&mut self.cx.image_releases);
        UiFrame {
            ops,
            surface,
            cursor,
            images,
            image_releases,
        }
    }

    fn refresh_semantics(&mut self) {
        self.semantics_dirty = false;
        let viewport = Rect::new(geometry::Point::ZERO, self.viewport);
        let diff =
            semantics_build::rebuild(&self.cx.tree, &mut self.semantics, self.focused, viewport);
        self.semantic_index = self
            .cx
            .tree
            .iter()
            .map(|(id, element)| (element.semantic_id, id))
            .collect();
        let focusables: Vec<(NodeId, Rect)> = self
            .cx
            .tree
            .iter()
            .filter(|(_, element)| element.is_focusable() && !element.visual.hidden)
            .map(|(id, element)| (id, element.geometry.window_rect))
            .collect();
        let mut ordered = focusables;
        ordered.sort_by(|(_, a), (_, b)| {
            a.min_y()
                .total_cmp(&b.min_y())
                .then(a.min_x().total_cmp(&b.min_x()))
        });
        self.focus_manager.set_focusables(&ordered);
        if !diff.updated.is_empty() || !diff.removed.is_empty() {
            self.pending_diff = Some(diff);
        }
    }

    /// The current semantic tree.
    pub fn semantic_tree(&mut self) -> &SemanticTree {
        if self.semantics_dirty {
            self.refresh_semantics();
        }
        &self.semantics
    }

    /// The semantic changes since the last call, for accessibility adapters.
    pub fn take_semantic_diff(&mut self) -> Option<SemanticDiff> {
        self.pending_diff.take()
    }

    /// The focused element's semantic id.
    pub fn focused_semantic_id(&self) -> Option<SemanticId> {
        self.focused
            .and_then(|node| self.cx.tree.get(node))
            .map(|element| element.semantic_id)
    }

    /// Performs a test, agent or accessibility action on a semantic node.
    pub fn perform(&mut self, id: SemanticId, action: &UiAction) -> Result<(), ActionError> {
        if self.semantics_dirty {
            self.refresh_semantics();
        }
        let node = *self
            .semantic_index
            .get(&id)
            .ok_or(ActionError::UnknownNode(id.0))?;
        let path = self.cx.tree.path(node);
        let element = self
            .cx
            .tree
            .get(node)
            .ok_or(ActionError::UnknownNode(id.0))?;
        if element.disabled {
            return Err(ActionError::Disabled(path));
        }
        let secure = element.semantics.secure;
        let center = element.geometry.window_rect.center();
        match action {
            UiAction::Click => {
                self.set_focus_if_focusable(node);
                self.click(node, true);
            }
            UiAction::DoubleClick => {
                self.click(node, true);
                let mut event = crate::handler::Dispatchable::Click(
                    crate::handler::ListenerKind::DoubleClick,
                    crate::event::Event::new(
                        crate::event::ClickEvent {
                            position: center,
                            count: 2,
                        },
                        true,
                    ),
                );
                self.bubble(node, &mut event);
            }
            UiAction::Hover => {
                let pointer = input::PointerEvent::mouse(center);
                self.update_hover(center, &pointer);
            }
            UiAction::Focus => self.set_focus(Some(node), true),
            UiAction::TypeText { text } => {
                if secure {
                    return Err(ActionError::Secure(path));
                }
                self.set_focus(Some(node), true);
                self.text_input(&input::TextInputEvent::Commit(text.clone()), true);
            }
            UiAction::PressKey { keybind } => self.press(*keybind),
            UiAction::Scroll { delta } => {
                let ancestors: Vec<NodeId> = self.cx.tree.ancestors(node).collect();
                self.scroll_nearest(&ancestors, *delta, true);
            }
            UiAction::Drag { to } => {
                let start = input::PointerEvent::mouse(center);
                let mut moved = start;
                moved.position = *to;
                moved.timestamp = start.timestamp + Duration::from_millis(100);
                let mut down = start;
                down.button = Some(input::PointerButton::Primary);
                self.pointer_down(&down);
                self.pointer_move(&moved);
                let mut up = moved;
                up.button = Some(input::PointerButton::Primary);
                self.pointer_up(&up);
            }
        }
        self.semantics_dirty = true;
        Ok(())
    }

    fn set_focus_if_focusable(&mut self, node: NodeId) {
        if self
            .cx
            .tree
            .get(node)
            .is_some_and(crate::node::ElementNode::is_focusable)
        {
            self.set_focus(Some(node), false);
        }
    }

    fn press(&mut self, keybind: Keybind) {
        let event = input::KeyboardEvent {
            key: keybind.key,
            code: input::PhysicalKey(0),
            text: match keybind.key {
                input::Key::Character(character) if keybind.modifiers == Modifiers::NONE => {
                    Some(character.to_string())
                }
                _ => None,
            },
            repeat: false,
            modifiers: keybind.modifiers,
        };
        self.key_down(&event, true);
        self.key_up(&event, true);
    }

    /// Introspection of the reactive graph and tree, for devtools.
    pub fn inspect_signals(&self) -> inspect::SignalGraphSnapshot {
        inspect::SignalGraphSnapshot {
            signals: self.cx.signal_count(),
            observers: self.cx.observer_count(),
            scopes: self.cx.scope_count(),
            components: Vec::new(),
        }
    }

    /// Introspection of the laid-out tree, for devtools.
    pub fn inspect_layout(&self) -> inspect::LayoutSnapshot {
        let mut nodes = Vec::new();
        let Some(root) = self.cx.tree.root() else {
            return inspect::LayoutSnapshot { nodes };
        };
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            let Some(element) = self.cx.tree.get(id) else {
                continue;
            };
            let rect = element.geometry.window_rect;
            nodes.push(inspect::LayoutNodeInfo {
                path: self.cx.tree.path(id),
                rect: [
                    rect.min_x(),
                    rect.min_y(),
                    rect.size.width,
                    rect.size.height,
                ],
                style: crate::util::style_summary::summarize(&element.style),
            });
            stack.extend(element.children.iter().rev().copied());
        }
        inspect::LayoutSnapshot { nodes }
    }

    /// Tools registered by mounted components with `cx.register_tool`.
    pub fn tools(&mut self) -> Vec<crate::tools::ToolInfo> {
        self.cx.live_tools()
    }

    /// Whether the tool list changed since the last call.
    pub fn tools_changed(&mut self) -> bool {
        self.cx.take_tools_changed()
    }

    /// Calls an app tool with JSON arguments; returns JSON text.
    pub fn call_tool(&mut self, name: &str, arguments: &str) -> Result<String, String> {
        let result = self.cx.call_tool(name, arguments);
        self.semantics_dirty = true;
        result
    }

    /// Clipboard operations requested by handlers since the last call.
    pub fn take_clipboard_requests(&mut self) -> Vec<crate::cx::ClipboardRequest> {
        std::mem::take(&mut self.cx.clipboard_requests)
    }

    /// Where an input method should place its candidate window, when a text field has focus,
    /// and whether the field is secure.
    pub fn text_input_request(&self) -> Option<(Rect, bool)> {
        let element = self.cx.tree.get(self.focused?)?;
        matches!(
            element.role,
            semantics::Role::TextField | semantics::Role::SearchField
        )
        .then_some((element.geometry.window_rect, element.semantics.secure))
    }

    /// Whether a pointer is down (long presses need polling).
    pub fn has_active_pointers(&self) -> bool {
        !self.dispatcher.pointers.is_empty()
    }

    /// Number of mounted elements.
    pub fn node_count(&self) -> usize {
        self.cx.tree.len()
    }

    /// Pending tasks spawned with `cx.spawn`.
    pub fn pending_tasks(&self) -> usize {
        self.cx.tasks.pending_count()
    }
}
