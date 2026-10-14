use std::time::Duration;

use geometry::{Point, Size, Vector};
use graphics::RecordingBackend;
use input::{Keybind, PointerButton, PointerEvent};
use runtime::InlineApp;
use semantics::{Query, SemanticId, SemanticNode, UiAction};
use taffy_layout::TaffyEngine;
use view::{ColorScheme, Ui, View};
use windowing::{WindowEvent, dpi::PhysicalSize};

const DEFAULT_SIZE: Size = Size::new(800.0, 600.0);

/// A headless app under test.
pub struct TestApp {
    app: InlineApp<RecordingBackend, TaffyEngine>,
}

impl TestApp {
    /// Mounts `root` in an 800×600 window.
    pub fn new(root: impl View) -> Self {
        Self::with_size(root, DEFAULT_SIZE.width, DEFAULT_SIZE.height)
    }

    /// Mounts `root` in a window of the given size.
    pub fn with_size(root: impl View, width: f32, height: f32) -> Self {
        match InlineApp::new(root, TaffyEngine::new(), Size::new(width, height)) {
            Ok(app) => Self { app },
            Err(error) => panic!("the recording backend cannot fail to start: {error}"),
        }
    }

    /// The UI, for anything the helpers do not cover.
    pub fn ui(&mut self) -> &mut Ui {
        self.app.ui()
    }

    /// The underlying inline app.
    pub fn inline(&mut self) -> &mut InlineApp<RecordingBackend, TaffyEngine> {
        &mut self.app
    }

    fn render(&mut self) {
        if let Err(error) = self.app.frame() {
            panic!("frame failed: {error}");
        }
    }

    /// Finds exactly one node, panicking with an explanation otherwise.
    pub fn find(&mut self, query: Query) -> SemanticNode {
        let tree = self.app.ui().semantic_tree();
        match query.find_one(tree) {
            Ok(id) => tree
                .get(id)
                .cloned()
                .unwrap_or_else(|| panic!("node {id:?} vanished")),
            Err(error) => panic!("{error}\n\ntree:\n{tree}"),
        }
    }

    /// Whether exactly one node matches.
    pub fn exists(&mut self, query: Query) -> bool {
        query.find_one(self.app.ui().semantic_tree()).is_ok()
    }

    /// How many nodes match.
    pub fn count(&mut self, query: Query) -> usize {
        query.find_all(self.app.ui().semantic_tree()).len()
    }

    /// The accessible name of the matching node.
    pub fn text_of(&mut self, query: Query) -> String {
        self.find(query).name.unwrap_or_default()
    }

    fn act(&mut self, id: SemanticId, action: UiAction) -> &mut Self {
        if let Err(error) = self.app.perform(id, &action) {
            panic!("{} failed: {error}", action.name());
        }
        self
    }

    /// Clicks the matching node.
    pub fn click(&mut self, query: Query) -> &mut Self {
        let id = self.find(query).id;
        self.act(id, UiAction::Click)
    }

    /// Double-clicks the matching node.
    pub fn double_click(&mut self, query: Query) -> &mut Self {
        let id = self.find(query).id;
        self.act(id, UiAction::DoubleClick)
    }

    /// Hovers the matching node.
    pub fn hover(&mut self, query: Query) -> &mut Self {
        let id = self.find(query).id;
        self.act(id, UiAction::Hover)
    }

    /// Focuses the matching node.
    pub fn focus(&mut self, query: Query) -> &mut Self {
        let id = self.find(query).id;
        self.act(id, UiAction::Focus)
    }

    /// Types `text` into the focused node.
    pub fn type_text(&mut self, text: &str) -> &mut Self {
        let Some(id) = self.app.ui().focused_semantic_id() else {
            panic!("type_text needs a focused node; click or focus a text field first");
        };
        self.act(id, UiAction::TypeText { text: text.into() })
    }

    /// Presses a shortcut such as `"Enter"` or `"Ctrl+S"` on the focused node.
    pub fn press_key(&mut self, shortcut: &str) -> &mut Self {
        let keybind: Keybind = shortcut
            .parse()
            .unwrap_or_else(|error| panic!("invalid shortcut {shortcut:?}: {error}"));
        let target = self
            .app
            .ui()
            .focused_semantic_id()
            .or_else(|| self.app.ui().semantic_tree().root());
        let Some(id) = target else {
            panic!("press_key needs a mounted tree");
        };
        self.act(id, UiAction::PressKey { keybind })
    }

    /// Scrolls the scroll container around the matching node by `dy` pixels.
    pub fn scroll(&mut self, query: Query, dy: f32) -> &mut Self {
        let id = self.find(query).id;
        self.act(
            id,
            UiAction::Scroll {
                delta: Vector::new(0.0, dy),
            },
        )
    }

    /// Drags the matching node to `(x, y)` in window coordinates.
    pub fn drag(&mut self, query: Query, x: f32, y: f32) -> &mut Self {
        let id = self.find(query).id;
        self.act(
            id,
            UiAction::Drag {
                to: Point::new(x, y),
            },
        )
    }

    fn pointer(&mut self, event: WindowEvent) -> &mut Self {
        if let Err(error) = self.app.handle(&event) {
            panic!("frame failed: {error}");
        }
        self
    }

    fn pointer_event(&self, x: f32, y: f32, button: Option<PointerButton>) -> PointerEvent {
        PointerEvent {
            button,
            timestamp: self.app.now(),
            ..PointerEvent::mouse(Point::new(x, y))
        }
    }

    /// Presses the primary button at `(x, y)`.
    pub fn pointer_down(&mut self, x: f32, y: f32) -> &mut Self {
        let event = self.pointer_event(x, y, Some(PointerButton::Primary));
        self.pointer(WindowEvent::PointerDown(event))
    }

    /// Moves the pointer to `(x, y)`.
    pub fn pointer_move(&mut self, x: f32, y: f32) -> &mut Self {
        let event = self.pointer_event(x, y, None);
        self.pointer(WindowEvent::PointerMove(event))
    }

    /// Releases the primary button at `(x, y)`.
    pub fn pointer_up(&mut self, x: f32, y: f32) -> &mut Self {
        let event = self.pointer_event(x, y, Some(PointerButton::Primary));
        self.pointer(WindowEvent::PointerUp(event))
    }

    /// Moves the clock forward, rendering every frame on the way.
    pub fn advance_ms(&mut self, millis: u64) -> &mut Self {
        if let Err(error) = self.app.advance(Duration::from_millis(millis)) {
            panic!("frame failed: {error}");
        }
        self
    }

    /// Advances until animations stop.
    pub fn settle(&mut self) -> &mut Self {
        if let Err(error) = self.app.settle() {
            panic!("frame failed: {error}");
        }
        self
    }

    /// Resizes the window.
    pub fn resize(&mut self, width: u32, height: u32) -> &mut Self {
        self.pointer(WindowEvent::Resized(PhysicalSize { width, height }))
    }

    /// Switches light/dark appearance.
    pub fn set_color_scheme(&mut self, scheme: ColorScheme) -> &mut Self {
        self.app.ui().set_color_scheme(scheme);
        self.render();
        self
    }

    /// Whether anything is still animating.
    pub fn is_animating(&self) -> bool {
        self.app.render_tree().is_animating()
    }

    /// The semantic tree as text.
    pub fn semantics_snapshot(&mut self) -> String {
        self.app.ui().semantic_tree().to_string()
    }

    /// The last frame's draw commands as text.
    pub fn frame_snapshot(&self) -> String {
        self.app
            .renderer()
            .last_frame()
            .map(|frame| frame.text.clone())
            .unwrap_or_default()
    }

    /// Element paths, rects and styles as text.
    pub fn layout_snapshot(&mut self) -> String {
        self.app
            .ui()
            .inspect_layout()
            .nodes
            .iter()
            .map(|node| {
                let [x, y, width, height] = node.rect.map(geometry::Length);
                format!("{} @{x},{y} {width}x{height} {}\n", node.path, node.style)
            })
            .collect()
    }

    /// Semantics, layout and frame together, for `insta::assert_snapshot!`.
    pub fn snapshot(&mut self) -> String {
        format!(
            "## semantics\n{}\n## layout\n{}\n## frame\n{}",
            self.semantics_snapshot(),
            self.layout_snapshot(),
            self.frame_snapshot()
        )
    }
}
