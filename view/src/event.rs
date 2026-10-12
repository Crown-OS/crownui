use std::ops::{Deref, DerefMut};

use geometry::{Point, Vector};

/// An event delivered to handlers, bubbling from the target to the root like the DOM.
#[derive(Debug, Clone, PartialEq)]
pub struct Event<T> {
    detail: T,
    propagation_stopped: bool,
    default_prevented: bool,
    synthetic: bool,
}

impl<T> Event<T> {
    pub(crate) fn new(detail: T, synthetic: bool) -> Self {
        Self {
            detail,
            propagation_stopped: false,
            default_prevented: false,
            synthetic,
        }
    }

    /// Stops the event from reaching ancestors.
    pub fn stop_propagation(&mut self) {
        self.propagation_stopped = true;
    }

    /// Cancels the framework's default action (focus on click, scroll on wheel, text insertion).
    pub fn prevent_default(&mut self) {
        self.default_prevented = true;
    }

    /// Whether a handler stopped propagation.
    pub fn is_propagation_stopped(&self) -> bool {
        self.propagation_stopped
    }

    /// Whether a handler prevented the default action.
    pub fn is_default_prevented(&self) -> bool {
        self.default_prevented
    }

    /// Whether the event was injected by a test, an agent (MCP) or assistive technology.
    pub fn is_synthetic(&self) -> bool {
        self.synthetic
    }

    /// The event payload.
    pub fn detail(&self) -> &T {
        &self.detail
    }
}

impl<T> Deref for Event<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.detail
    }
}

impl<T> DerefMut for Event<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.detail
    }
}

/// A click, double click or context-menu request (DOM `click`, `dblclick`, `contextmenu`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClickEvent {
    /// Position relative to the target's top-left corner.
    pub position: Point,
    /// Consecutive clicks: 1, 2 or 3.
    pub count: u32,
}

/// A press held in place (no DOM equivalent).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LongPressEvent {
    /// Position relative to the target.
    pub position: Point,
}

/// A drag phase (DOM `dragstart`, `drag`, `dragend`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragEvent {
    /// Pointer position relative to the target.
    pub position: Point,
    /// Movement since the previous drag event.
    pub delta: Vector,
    /// Movement since the drag started.
    pub total: Vector,
    /// Velocity in logical pixels per second; set on `on_drag_end`.
    pub velocity: Vector,
}

/// A pinch phase from a touchpad (no DOM equivalent).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PinchEvent {
    /// Scale relative to the start of the gesture.
    pub scale: f32,
    /// Rotation in degrees since the previous event.
    pub rotation: f32,
}

/// Text changed by typing (DOM `input`) or committed (DOM `change`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputEvent {
    /// The full new value.
    pub value: String,
}

/// Focus moved to or from the target (DOM `focus`, `blur`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusEvent {
    /// Whether focus arrived by keyboard (CSS `:focus-visible`).
    pub visible: bool,
}

/// A scroll container's offset changed (DOM `scroll`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollEvent {
    /// New content offset.
    pub offset: Vector,
}

/// A drag from any app moving over the target (HTML `dragenter` / `dragover`). Call
/// [`accept`](Self::accept) to let it drop here; positions are in window coordinates.
#[derive(Debug, Clone, PartialEq)]
pub struct DragOverEvent {
    /// Pointer position in window coordinates.
    pub position: Point,
    /// MIME types the dragged data can be read as.
    pub mime_types: Vec<String>,
    accepted: Option<String>,
}

impl DragOverEvent {
    pub(crate) fn new(position: Point, mime_types: Vec<String>) -> Self {
        Self {
            position,
            mime_types,
            accepted: None,
        }
    }

    /// Whether the drag can be read as `mime_type`.
    pub fn offers(&self, mime_type: &str) -> bool {
        self.mime_types.iter().any(|offered| offered == mime_type)
    }

    /// Lets the drag drop here, delivered as `mime_type` to `on_drop`.
    pub fn accept(&mut self, mime_type: impl Into<String>) {
        self.accepted = Some(mime_type.into());
    }

    pub(crate) fn accepted(&self) -> Option<&str> {
        self.accepted.as_deref()
    }
}

/// The drag left the target without dropping on it (HTML `dragleave`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DragLeaveEvent;

/// Data dropped on the target (HTML `drop`), read in the MIME type a `dragover` handler accepted.
#[derive(Debug, Clone, PartialEq)]
pub struct DropEvent {
    /// Drop position in window coordinates.
    pub position: Point,
    /// The accepted MIME type.
    pub mime_type: String,
    /// The dropped bytes.
    pub data: Vec<u8>,
}
