use input::{KeyboardEvent, Modifiers, PointerEvent, TextInputEvent, TouchpadGesture, WheelEvent};

use crate::dpi::{PhysicalSize, ScaleFactor};

/// Everything a window can report, with DOM-style names for input.
#[derive(Debug, Clone, PartialEq)]
pub enum WindowEvent {
    /// The platform started or stopped showing the window, or shows it scaled (an overview).
    Visibility {
        /// Whether the window is on screen.
        visibility: crate::Visibility,
        /// Scale the window is shown at; 1 normally, smaller in an overview.
        thumbnail_scale: f32,
    },
    /// The drawable size changed.
    Resized(PhysicalSize),
    /// The surface scale changed; a `Resized` follows.
    ScaleFactorChanged(ScaleFactor),
    /// The user asked to close the window.
    CloseRequested,
    /// The platform is ready for a new frame.
    RedrawRequested,
    /// Keyboard focus entered (`true`) or left (`false`).
    Focused(bool),
    /// Held modifiers changed.
    ModifiersChanged(Modifiers),
    /// A key went down (or repeated).
    KeyDown(KeyboardEvent),
    /// A key went up.
    KeyUp(KeyboardEvent),
    /// A pointer button or touch contact went down.
    PointerDown(PointerEvent),
    /// A pointer button or touch contact went up.
    PointerUp(PointerEvent),
    /// A pointer moved.
    PointerMove(PointerEvent),
    /// A pointer entered the window.
    PointerEnter(PointerEvent),
    /// A pointer left the window.
    PointerLeave(PointerEvent),
    /// The platform took the pointer away mid-gesture.
    PointerCancel(PointerEvent),
    /// Wheel or touchpad scrolling.
    Wheel(WheelEvent),
    /// Input-method composition or commit.
    TextInput(TextInputEvent),
    /// Multi-finger touchpad gesture.
    TouchpadGesture(TouchpadGesture),
    /// The clipboard's text, answering [`Platform::request_clipboard_text`](crate::Platform::request_clipboard_text).
    ClipboardText(String),
    /// A drag from any client entered the window (HTML `dragenter`).
    DragEnter(crate::DragOffer),
    /// The drag moved within the window, in window-local logical pixels (HTML `dragover`).
    DragOver(geometry::Point),
    /// The drag left the window without dropping (HTML `dragleave`).
    DragLeave,
    /// The accepted drag was dropped and its data read (HTML `drop`).
    Drop(crate::DroppedData),
    /// A drag this app started through [`Platform::start_drag`](crate::Platform::start_drag) ended.
    DragEnded(crate::DragOutcome),
    /// The user activated the icon of an application that has no windows (see
    /// [`Platform::activate_app_icon`](crate::Platform::activate_app_icon)): start the app `app_id`.
    LaunchApp(String),
}
