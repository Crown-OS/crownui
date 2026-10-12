use std::ops::{Deref, DerefMut};

use geometry::Size;
use graphics::DrawList;
use input::Keybind;
use motion::{Curve, Transaction};
use reactive::{Runtime, ScopeId, Signal, Store};
use semantics::{Role, SemanticId};
use smallvec::SmallVec;

use crate::{
    NodeId,
    animation_frames::AnimationFrames,
    app_icons::AppIcons,
    color_scheme::ColorScheme,
    element::Kind,
    handler::{Dispatchable, Listener, ListenerKind},
    modifier::{Modifier, SemanticProp, Variant, VariantProp},
    node::{CanvasContent, ElementNode, NodeContent, TextContent},
    prop::{IntoProp, Prop},
    style_prop::StyleProp,
    tasks::Tasks,
    tree::ElementTree,
    view::ViewSeq,
};

/// A deferred change produced by reactive effects and applied to the tree after each flush.
pub(crate) enum Command {
    Style(NodeId, StyleProp),
    Text(NodeId, String),
    Label(NodeId, String),
    Value(NodeId, String),
    Disabled(NodeId, bool),
    Checked(NodeId, bool),
    VariantProp(NodeId, usize, usize, StyleProp),
    Canvas(NodeId, DrawList),
    RefreshVariants(NodeId),
    Rebuild(NodeId),
    ReleaseImage(graphics::ImageId),
}

/// The context every build function, handler and binding receives. It owns the reactive
/// runtime (it derefs to [`Runtime`], so `signal.get(cx)` works) and the element tree.
pub struct Cx {
    runtime: Runtime,
    pub(crate) tree: ElementTree,
    pub(crate) commands: Store<Vec<Command>>,
    pub(crate) transaction: Option<Transaction>,
    pub(crate) frame_transaction: Option<Transaction>,
    pub(crate) color_scheme: Signal<ColorScheme>,
    pub(crate) tasks: Tasks,
    pub(crate) focus_request: Option<NodeId>,
    pub(crate) deferred: Vec<(NodeId, Dispatchable)>,
    pub(crate) image_uploads: Vec<ImageUpload>,
    pub(crate) image_releases: Vec<graphics::ImageId>,
    next_image: u64,
    pub(crate) tools: crate::tools::ToolRegistry,
    pub(crate) clipboard_requests: Vec<ClipboardRequest>,
    pub(crate) animation_frames: AnimationFrames,
    pub(crate) idle_inhibited: bool,
    pub(crate) app_icons: AppIcons,
    pub(crate) capabilities: Signal<windowing::Capabilities>,
    pub(crate) scale_factor: Signal<f32>,
    pub(crate) drag_request: Option<windowing::DragSource>,
    pub(crate) drag_end: Option<DragEnd>,
}

type DragEnd = Box<dyn FnOnce(&mut Cx, windowing::DragOutcome)>;

/// A clipboard operation for the platform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardRequest {
    /// Put text on the clipboard.
    Copy(String),
    /// Read the clipboard; the text arrives as a paste into the focused element.
    Paste,
}

/// Pixels waiting to reach the renderer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageUpload {
    /// Id the image is drawn by.
    pub id: graphics::ImageId,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Straight-alpha RGBA8 pixels.
    pub rgba: Vec<u8>,
}

impl Deref for Cx {
    type Target = Runtime;
    fn deref(&self) -> &Runtime {
        &self.runtime
    }
}

impl DerefMut for Cx {
    fn deref_mut(&mut self) -> &mut Runtime {
        &mut self.runtime
    }
}

fn semantic_id_for(parent: SemanticId, key: u64, name: &str) -> SemanticId {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    parent.0.hash(&mut hasher);
    key.hash(&mut hasher);
    name.hash(&mut hasher);
    SemanticId(hasher.finish())
}

impl Cx {
    pub(crate) fn new() -> Self {
        let mut runtime = Runtime::new();
        let commands = runtime.store(Vec::new());
        let color_scheme = runtime.provide(ColorScheme::Light);
        let capabilities = runtime.signal(windowing::Capabilities::default());
        let scale_factor = runtime.signal(1.0);
        Self {
            runtime,
            tree: ElementTree::default(),
            commands,
            transaction: None,
            frame_transaction: None,
            color_scheme,
            tasks: Tasks::default(),
            focus_request: None,
            deferred: Vec::new(),
            image_uploads: Vec::new(),
            image_releases: Vec::new(),
            next_image: 1,
            tools: crate::tools::ToolRegistry::default(),
            clipboard_requests: Vec::new(),
            animation_frames: AnimationFrames::default(),
            idle_inhibited: false,
            app_icons: AppIcons::default(),
            capabilities,
            scale_factor,
            drag_request: None,
            drag_end: None,
        }
    }

    /// The element tree.
    pub fn tree(&self) -> &ElementTree {
        &self.tree
    }

    /// What the platform the app runs on supports (materials, layer shell, idle inhibition …), so
    /// a component can pick a fallback instead of relying on something that is not there.
    pub fn platform_capabilities(&self) -> Signal<windowing::Capabilities> {
        self.capabilities
    }

    /// Device pixels per logical pixel of the window, so pixel art can be made at the size it shows.
    pub fn scale_factor(&self) -> Signal<f32> {
        self.scale_factor
    }

    /// Hands `source` to the platform to drag out of the window, the compositor moving its icon
    /// with the pointer; call it from a `dragstart` or pointer-down handler while the button is
    /// held. `on_end` learns whether a target took it. Returns `false`, dropping both, while an
    /// earlier drag is still in flight.
    pub fn start_drag(
        &mut self,
        source: windowing::DragSource,
        on_end: impl FnOnce(&mut Cx, windowing::DragOutcome) + 'static,
    ) -> bool {
        if self.drag_end.is_some() {
            return false;
        }
        self.drag_request = Some(source);
        self.drag_end = Some(Box::new(on_end));
        true
    }

    /// The current color scheme signal (drives `dark:` variants).
    pub fn color_scheme(&self) -> Signal<ColorScheme> {
        self.color_scheme
    }

    /// Runs `change` so that every resulting property change animates with `curve`
    /// (SwiftUI's `withAnimation`).
    pub fn with_animation(&mut self, curve: impl Into<Curve>, change: impl FnOnce(&mut Self)) {
        let transaction = Transaction::new(curve);
        let previous = self.transaction.replace(transaction);
        change(self);
        self.apply_pending();
        self.transaction = previous;
        self.frame_transaction = Some(transaction);
    }

    /// Moves keyboard focus to `node` after the current event.
    pub fn request_focus(&mut self, node: NodeId) {
        self.focus_request = Some(node);
    }

    /// Puts `text` on the system clipboard.
    pub fn copy_to_clipboard(&mut self, text: impl Into<String>) {
        self.clipboard_requests
            .push(ClipboardRequest::Copy(text.into()));
    }

    /// Keeps the system from idling (screen blanking, sleep) while the window is visible, or lets
    /// it idle again. Released with the window, so a crash cannot keep the machine awake.
    pub fn set_idle_inhibited(&mut self, inhibited: bool) {
        self.idle_inhibited = inhibited;
    }

    /// Requests the clipboard's text; it is delivered as a paste into the focused element.
    pub fn paste_from_clipboard(&mut self) {
        self.clipboard_requests.push(ClipboardRequest::Paste);
    }

    /// Fires DOM `input` on `node` after the current handler (used by text fields).
    pub fn emit_input(&mut self, node: NodeId, value: impl Into<String>) {
        self.defer_input(node, ListenerKind::Input, value.into());
    }

    /// Fires DOM `change` on `node` after the current handler.
    pub fn emit_change(&mut self, node: NodeId, value: impl Into<String>) {
        self.defer_input(node, ListenerKind::Change, value.into());
    }

    /// Fires DOM `submit` on `node` after the current handler.
    pub fn emit_submit(&mut self, node: NodeId, value: impl Into<String>) {
        self.defer_input(node, ListenerKind::Submit, value.into());
    }

    fn defer_input(&mut self, node: NodeId, kind: ListenerKind, value: String) {
        let event = crate::event::Event::new(crate::event::InputEvent { value }, false);
        self.deferred.push((node, Dispatchable::Input(kind, event)));
    }

    /// Queues straight-alpha RGBA8 pixels for the renderer and returns the id to draw them by.
    pub fn register_image(&mut self, width: u32, height: u32, rgba: Vec<u8>) -> graphics::ImageId {
        let id = graphics::ImageId(self.next_image);
        self.next_image += 1;
        self.image_uploads.push(ImageUpload {
            id,
            width,
            height,
            rgba,
        });
        id
    }

    /// Frees an image from [`Cx::register_image`] once nothing on screen draws it anymore,
    /// exiting nodes included. Drawing it after this call draws nothing.
    pub fn release_image(&mut self, image: graphics::ImageId) {
        match self
            .image_uploads
            .iter()
            .position(|upload| upload.id == image)
        {
            Some(pending) => {
                self.image_uploads.swap_remove(pending);
            }
            None => self.image_releases.push(image),
        }
    }

    /// Releases `image` (see [`Cx::release_image`]) when the current scope ends, so an image
    /// lives exactly as long as the component that registered it.
    pub fn release_image_on_cleanup(&mut self, image: graphics::ImageId) {
        let commands = self.commands;
        self.on_cleanup(move |runtime| {
            commands.update(runtime, |queue| queue.push(Command::ReleaseImage(image)));
        });
    }

    /// Makes `node` draw `image` at `natural_size` unless styled otherwise.
    pub fn set_image(&mut self, node: NodeId, image: graphics::ImageId, natural_size: Size) {
        if let Some(element) = self.tree.get_mut(node) {
            element.role = Role::Image;
            element.content = NodeContent::Image(crate::node::ImageContent {
                image,
                natural_size,
            });
            element.dirty.measure = true;
            element.dirty.content = true;
        }
    }

    /// Mounts a view as a child of nothing; the caller attaches the returned root.
    pub fn build(&mut self, view: impl crate::View) -> NodeId {
        view.build(self)
    }

    /// Mounts `children` and makes them `node`'s children, replacing previous ones.
    pub fn set_children(&mut self, node: NodeId, children: impl ViewSeq) {
        let mut built = Vec::new();
        children.build_into(self, &mut built);
        self.tree.set_children(node, built);
    }

    /// Makes already-built nodes `node`'s children.
    pub fn adopt_children(&mut self, node: NodeId, children: Vec<NodeId>) {
        self.tree.set_children(node, children);
    }

    /// Binds a style property of `node` to signals, as a kind's default.
    pub fn bind_style(
        &mut self,
        node: NodeId,
        compute: impl FnMut(&mut Runtime) -> StyleProp + 'static,
    ) {
        if let Some(first) =
            self.track_value(Box::new(compute), move |prop| Command::Style(node, prop))
        {
            self.apply_style(node, &first);
        }
    }

    /// Binds the accessible value (e.g. a text field's contents) to signals.
    pub fn bind_value(
        &mut self,
        node: NodeId,
        compute: impl FnMut(&mut Runtime) -> String + 'static,
    ) {
        let first = self.track_value(Box::new(compute), move |value| Command::Value(node, value));
        if let Some(element) = self.tree.get_mut(node) {
            element.semantics.value = first;
        }
    }

    /// Binds the checked state reported to assistive technology.
    pub fn bind_checked(
        &mut self,
        node: NodeId,
        compute: impl FnMut(&mut Runtime) -> bool + 'static,
    ) {
        let first = self.track_value(Box::new(compute), move |checked| {
            Command::Checked(node, checked)
        });
        self.set_checked(node, first);
    }

    /// A signal that follows `node`'s laid-out size (updated after every layout pass).
    pub fn observe_size(&mut self, node: NodeId) -> Signal<Size> {
        if let Some(existing) = self.tree.get(node).and_then(|element| element.size_signal) {
            return existing;
        }
        let signal = self.runtime.signal(Size::ZERO);
        if let Some(element) = self.tree.get_mut(node) {
            element.size_signal = Some(signal);
        }
        signal
    }

    /// A signal that follows whether `node` is disabled (by its `disabled` modifier).
    pub fn observe_disabled(&mut self, node: NodeId) -> Signal<bool> {
        if let Some(existing) = self
            .tree
            .get(node)
            .and_then(|element| element.disabled_signal)
        {
            return existing;
        }
        let initial = self.tree.get(node).is_some_and(|element| element.disabled);
        let signal = self.runtime.signal(initial);
        if let Some(element) = self.tree.get_mut(node) {
            element.disabled_signal = Some(signal);
        }
        signal
    }

    fn set_disabled(&mut self, node: NodeId, disabled: bool) {
        let Some(element) = self.tree.get_mut(node) else {
            return;
        };
        element.disabled = disabled;
        if let Some(signal) = element.disabled_signal {
            signal.set(&mut self.runtime, disabled);
        }
    }

    /// Sets the ARIA role a kind implies.
    pub fn set_role(&mut self, node: NodeId, role: Role) {
        if let Some(element) = self.tree.get_mut(node) {
            element.role = role;
        }
    }

    /// Applies a default style that the element's modifiers may override.
    pub fn set_default_style(&mut self, node: NodeId, prop: StyleProp) {
        self.apply_style(node, &prop);
    }

    /// Adds a listener on behalf of a kind (e.g. a toggle flipping its signal on click).
    pub fn add_listener(&mut self, node: NodeId, listener: Listener) {
        if let Some(element) = self.tree.get_mut(node) {
            element.listeners.push(listener);
        }
    }

    /// Sets the checked state reported to assistive technology.
    pub fn set_checked(&mut self, node: NodeId, checked: Option<bool>) {
        if let Some(element) = self.tree.get_mut(node) {
            element.semantics.checked = checked;
        }
    }

    /// Makes `node` display text, recomputed when the signals it reads change.
    pub fn set_text(&mut self, node: NodeId, text: impl IntoProp<String>) {
        let initial = match text.into_prop() {
            Prop::Static(text) => text,
            Prop::Dynamic(compute) => self
                .track_value(compute, move |text| Command::Text(node, text))
                .unwrap_or_default(),
        };
        if let Some(element) = self.tree.get_mut(node) {
            element.role = Role::Text;
            element.content = NodeContent::Text(TextContent {
                text: initial,
                layout: None,
            });
            element.dirty.measure = true;
            element.dirty.content = true;
        }
    }

    /// Makes `node` draw with `painter`, re-run when its size or the signals it reads change.
    pub fn set_canvas(
        &mut self,
        node: NodeId,
        mut painter: impl FnMut(&mut Runtime, Size, &mut DrawList) + 'static,
    ) {
        let size = self.runtime.signal(Size::ZERO);
        let commands = self.commands;
        if let Some(element) = self.tree.get_mut(node) {
            element.content = NodeContent::Canvas(CanvasContent {
                size,
                list: DrawList::new(),
            });
        }
        self.runtime.effect(move |runtime| {
            let current = size.get(runtime);
            let mut list = DrawList::new();
            painter(runtime, current, &mut list);
            commands.update(runtime, |queue| queue.push(Command::Canvas(node, list)));
        });
    }

    /// Creates a node for `kind`, mounts it and applies `modifiers` in order.
    pub(crate) fn mount_element<K: Kind>(
        &mut self,
        kind: K,
        modifiers: SmallVec<[Modifier; 6]>,
    ) -> NodeId {
        let scope = self.runtime.current_scope();
        let node = self.tree.insert(ElementNode::new(None, scope, K::NAME));
        kind.mount(self, node);
        let mut has_variants = false;
        for modifier in modifiers {
            has_variants |= matches!(modifier, Modifier::Variant(..));
            self.apply_modifier(node, modifier);
        }
        if has_variants {
            self.watch_color_scheme(node);
        }
        self.refresh_visual(node);
        node
    }

    fn apply_modifier(&mut self, node: NodeId, modifier: Modifier) {
        match modifier {
            Modifier::Style(prop) => self.apply_style(node, &prop),
            Modifier::Bound(compute) => {
                if let Some(first) =
                    self.track_value(compute, move |prop| Command::Style(node, prop))
                {
                    self.apply_style(node, &first);
                }
            }
            Modifier::Variant(variant, props) => self.add_variant(node, variant, props),
            Modifier::Enter(props) => {
                if let Some(element) = self.tree.get_mut(node) {
                    element.enter = Some(props);
                }
            }
            Modifier::Exit(props) => {
                if let Some(element) = self.tree.get_mut(node) {
                    element.exit = Some(props);
                }
            }
            Modifier::Listener(listener) => self.add_listener(node, listener),
            Modifier::Transition(curve) => {
                if let Some(element) = self.tree.get_mut(node) {
                    element.transition = Some(curve);
                }
            }
            Modifier::Semantic(prop) => self.apply_semantic(node, prop),
            Modifier::Disabled(prop) => {
                let initial = self.bind_prop(node, prop, Command::Disabled);
                self.set_disabled(node, initial);
            }
            Modifier::Shortcut(keybind) => self.set_shortcut(node, keybind),
            Modifier::Key(key) => {
                if let Some(element) = self.tree.get_mut(node) {
                    element.key = Some(key);
                }
            }
        }
    }

    fn add_variant(&mut self, node: NodeId, variant: Variant, props: Vec<VariantProp>) {
        let Some(slot) = self.tree.get(node).map(|element| element.variants.len()) else {
            return;
        };
        let mut resolved = Vec::with_capacity(props.len());
        for prop in props {
            match prop {
                VariantProp::Literal(prop) => resolved.push(prop),
                VariantProp::Bound(compute) => {
                    let index = resolved.len();
                    let first = self.track_value(compute, move |prop| {
                        Command::VariantProp(node, slot, index, prop)
                    });
                    match first {
                        Some(prop) if prop.is_visual() || prop.impact().text => resolved.push(prop),
                        Some(prop) => tracing::warn!(
                            ?prop,
                            "CUI0201: layout properties are ignored in style variants"
                        ),
                        None => {}
                    }
                }
            }
        }
        if let Some(element) = self.tree.get_mut(node) {
            element.variants.push((variant, resolved));
        }
    }

    fn set_variant_prop(&mut self, node: NodeId, slot: usize, index: usize, prop: StyleProp) {
        if !(prop.is_visual() || prop.impact().text) {
            tracing::warn!(
                ?prop,
                "CUI0201: layout properties are ignored in style variants"
            );
            return;
        }
        if let Some(target) = self
            .tree
            .get_mut(node)
            .and_then(|element| element.variants.get_mut(slot))
            .and_then(|(_, props)| props.get_mut(index))
        {
            *target = prop;
        }
    }

    fn set_shortcut(&mut self, node: NodeId, keybind: Keybind) {
        if let Some(element) = self.tree.get_mut(node) {
            element.shortcut = Some(keybind);
        }
    }

    fn apply_semantic(&mut self, node: NodeId, prop: SemanticProp) {
        match prop {
            SemanticProp::Label(label) => {
                let initial = self.bind_prop(node, label, Command::Label);
                if let Some(element) = self.tree.get_mut(node) {
                    element.semantics.label = Some(initial);
                }
            }
            SemanticProp::Value(value) => {
                let initial = self.bind_prop(node, value, Command::Value);
                if let Some(element) = self.tree.get_mut(node) {
                    element.semantics.value = Some(initial);
                }
            }
            SemanticProp::Description(description) => {
                if let Some(element) = self.tree.get_mut(node) {
                    element.semantics.description = Some(description);
                }
            }
            SemanticProp::Role(role) => {
                if let Some(element) = self.tree.get_mut(node) {
                    element.semantics.role = Some(role);
                }
            }
            SemanticProp::Focusable(focusable) => {
                if let Some(element) = self.tree.get_mut(node) {
                    element.semantics.focusable = Some(focusable);
                }
            }
            SemanticProp::Secure => {
                if let Some(element) = self.tree.get_mut(node) {
                    element.semantics.secure = true;
                }
            }
        }
    }

    fn bind_prop<T: Default + 'static>(
        &mut self,
        node: NodeId,
        prop: Prop<T>,
        command: fn(NodeId, T) -> Command,
    ) -> T {
        match prop {
            Prop::Static(value) => value,
            Prop::Dynamic(compute) => self
                .track_value(compute, move |value| command(node, value))
                .unwrap_or_default(),
        }
    }

    /// Evaluates `compute` once now (tracked) and queues `command` with each later result.
    fn track_value<T: 'static>(
        &mut self,
        mut compute: Box<dyn FnMut(&mut Runtime) -> T>,
        command: impl Fn(T) -> Command + 'static,
    ) -> Option<T> {
        let commands = self.commands;
        let first = self.runtime.store(None::<T>);
        let mut primed = false;
        self.runtime.effect(move |runtime| {
            let value = compute(runtime);
            if std::mem::replace(&mut primed, true) {
                commands.update(runtime, |queue| queue.push(command(value)));
            } else {
                first.update(runtime, |slot| *slot = Some(value));
            }
        });
        first.update(&mut self.runtime, Option::take)
    }

    fn watch_color_scheme(&mut self, node: NodeId) {
        let has_dark = self.tree.get(node).is_some_and(|element| {
            element
                .variants
                .iter()
                .any(|(variant, _)| *variant == Variant::Dark)
        });
        if !has_dark {
            return;
        }
        let commands = self.commands;
        let scheme = self.color_scheme;
        let mut primed = false;
        self.runtime.effect(move |runtime| {
            scheme.get(runtime);
            if std::mem::replace(&mut primed, true) {
                commands.update(runtime, |queue| queue.push(Command::RefreshVariants(node)));
            }
        });
    }

    pub(crate) fn apply_style(&mut self, node: NodeId, prop: &StyleProp) {
        let impact = prop.impact();
        let Some(element) = self.tree.get_mut(node) else {
            return;
        };
        prop.apply(&mut element.style);
        if impact.layout {
            element.dirty.style = true;
        }
        if impact.text {
            element.text = element.style.text.clone();
            self.mark_text_subtree(node);
        }
        if impact.visual {
            self.refresh_visual(node);
        }
    }

    fn mark_text_subtree(&mut self, node: NodeId) {
        let mut stack = vec![node];
        while let Some(current) = stack.pop() {
            let Some(element) = self.tree.get_mut(current) else {
                continue;
            };
            element.dirty.measure = true;
            element.dirty.content = true;
            stack.extend(element.children.iter().copied());
        }
    }

    /// Recomputes a node's effective visual from its base style and active variants.
    pub(crate) fn refresh_visual(&mut self, node: NodeId) {
        let dark = self
            .color_scheme
            .with_untracked(&self.runtime, |scheme| *scheme == ColorScheme::Dark);
        let Some(element) = self.tree.get_mut(node) else {
            return;
        };
        let state = element.interaction;
        let active = |variant: Variant| match variant {
            Variant::Hover => state.hovered,
            Variant::Active => state.pressed,
            Variant::Focus => state.focused,
            Variant::FocusVisible => state.focus_visible,
            Variant::Disabled => element.disabled,
            Variant::Dark => dark,
        };
        let mut style = element.style.clone();
        let mut text_changed = false;
        for (variant, props) in &element.variants {
            if active(*variant) {
                for prop in props {
                    text_changed |= prop.impact().text;
                    prop.apply(&mut style);
                }
            }
        }
        if element.visual != style.visual {
            element.visual = style.visual;
            element.dirty.visual = true;
        }
        if style.text != element.text {
            element.text = style.text;
            text_changed = true;
        }
        if text_changed {
            self.mark_text_subtree(node);
        }
    }

    /// Makes the root fill the window unless it has an explicit size, like a page's body.
    pub(crate) fn fill_viewport(&mut self, root: NodeId) {
        let Some(element) = self.tree.get(root) else {
            return;
        };
        let layout = &element.style.layout;
        let fill = |dimension: layouting::Dimension| dimension == layouting::Dimension::Auto;
        let (fill_width, fill_height) = (fill(layout.width), fill(layout.height));
        let full = layouting::Dimension::Percent(1.0);
        if fill_width {
            self.apply_style(
                root,
                &StyleProp::Size(crate::style_prop::SizeProperty::Width, full),
            );
        }
        if fill_height {
            self.apply_style(
                root,
                &StyleProp::Size(crate::style_prop::SizeProperty::Height, full),
            );
        }
    }

    /// Applies every queued command until effects stop producing new ones.
    pub(crate) fn apply_pending(&mut self) {
        loop {
            if let Err(error) = self.runtime.flush() {
                tracing::error!("{error}");
            }
            let commands = self.commands.update(&mut self.runtime, std::mem::take);
            if commands.is_empty() {
                break;
            }
            for command in commands {
                self.apply_command(command);
            }
        }
    }

    fn apply_command(&mut self, command: Command) {
        match command {
            Command::Style(node, prop) => self.apply_style(node, &prop),
            Command::Text(node, text) => {
                if let Some(element) = self.tree.get_mut(node)
                    && let NodeContent::Text(content) = &mut element.content
                    && content.text != text
                {
                    content.text = text;
                    element.dirty.measure = true;
                    element.dirty.content = true;
                }
            }
            Command::Label(node, label) => {
                if let Some(element) = self.tree.get_mut(node) {
                    element.semantics.label = Some(label);
                }
            }
            Command::Value(node, value) => {
                if let Some(element) = self.tree.get_mut(node) {
                    element.semantics.value = Some(value);
                }
            }
            Command::Disabled(node, disabled) => {
                self.set_disabled(node, disabled);
                self.refresh_visual(node);
            }
            Command::Canvas(node, list) => {
                if let Some(element) = self.tree.get_mut(node)
                    && let NodeContent::Canvas(canvas) = &mut element.content
                {
                    canvas.list = list;
                    element.dirty.content = true;
                }
            }
            Command::Checked(node, checked) => self.set_checked(node, Some(checked)),
            Command::VariantProp(node, slot, index, prop) => {
                self.set_variant_prop(node, slot, index, prop);
                self.refresh_visual(node);
            }
            Command::RefreshVariants(node) => self.refresh_visual(node),
            Command::Rebuild(node) => self.rebuild_dynamic(node),
            Command::ReleaseImage(image) => self.release_image(image),
        }
    }

    /// Assigns stable semantic ids top-down from keys and sibling positions.
    pub(crate) fn assign_semantic_ids(&mut self) {
        let Some(root) = self.tree.root() else {
            return;
        };
        let mut stack = vec![(root, SemanticId(0), 0_u64)];
        while let Some((node, parent_id, position)) = stack.pop() {
            let Some(element) = self.tree.get_mut(node) else {
                continue;
            };
            let key = element.key.unwrap_or(position);
            element.semantic_id = semantic_id_for(parent_id, key, element.name);
            let id = element.semantic_id;
            stack.extend(
                element
                    .children
                    .iter()
                    .enumerate()
                    .map(|(index, child)| (*child, id, index as u64)),
            );
        }
    }

    pub(crate) fn scope_of(&self, node: NodeId) -> Option<ScopeId> {
        self.tree.get(node).map(|element| element.scope)
    }
}
