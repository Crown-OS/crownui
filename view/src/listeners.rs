use input::{KeyboardEvent, Keybind, PointerEvent, TextInputEvent, WheelEvent};

use crate::{
    Cx,
    event::{
        ClickEvent, DragEvent, DragLeaveEvent, DragOverEvent, DropEvent, Event, FocusEvent, InputEvent, LongPressEvent,
        PinchEvent, ScrollEvent,
    },
    handler::Listener,
    modifier::Modifier,
    styled::Styled,
};

macro_rules! listener_methods {
    ($($(#[$doc:meta])* $name:ident($event:ty) => $variant:ident;)+) => {
        $($(#[$doc])* fn $name(self, handler: impl FnMut(&mut Cx, &mut Event<$event>) + 'static) -> Self {
            self.modifier(Modifier::Listener(Listener::$variant(Box::new(handler))))
        })+
    };
}

/// Event listeners named after DOM events. Handlers get the context and a bubbling event with
/// `stop_propagation()` and `prevent_default()`.
pub trait Listeners: Styled {
    listener_methods! {
        /// DOM `click` (primary button or tap; also fired by Enter/Space on a focused element).
        on_click(ClickEvent) => Click;
        /// DOM `dblclick`.
        on_double_click(ClickEvent) => DoubleClick;
        /// DOM `contextmenu` (secondary click).
        on_context_menu(ClickEvent) => ContextMenu;
        /// A press held in place; a touch extension the DOM lacks.
        on_long_press(LongPressEvent) => LongPress;
        /// DOM `pointerdown`.
        on_pointer_down(PointerEvent) => PointerDown;
        /// DOM `pointerup`.
        on_pointer_up(PointerEvent) => PointerUp;
        /// DOM `pointermove`.
        on_pointer_move(PointerEvent) => PointerMove;
        /// DOM `pointerenter`.
        on_pointer_enter(PointerEvent) => PointerEnter;
        /// DOM `pointerleave`.
        on_pointer_leave(PointerEvent) => PointerLeave;
        /// DOM `pointercancel`.
        on_pointer_cancel(PointerEvent) => PointerCancel;
        /// DOM `wheel`.
        on_wheel(WheelEvent) => Wheel;
        /// DOM `scroll`.
        on_scroll(ScrollEvent) => Scroll;
        /// DOM `keydown` (focused element and its ancestors).
        on_key_down(KeyboardEvent) => KeyDown;
        /// DOM `keyup`.
        on_key_up(KeyboardEvent) => KeyUp;
        /// DOM `input`: every edit of a text field.
        on_input(InputEvent) => Input;
        /// DOM `change`: a committed value.
        on_change(InputEvent) => Change;
        /// DOM `submit`: Enter in a text field.
        on_submit(InputEvent) => Submit;
        /// DOM `focus`.
        on_focus(FocusEvent) => Focus;
        /// DOM `blur`.
        on_blur(FocusEvent) => Blur;
        /// DOM `dragstart`.
        on_drag_start(DragEvent) => DragStart;
        /// DOM `drag`.
        on_drag(DragEvent) => Drag;
        /// DOM `dragend`, carrying the release velocity.
        on_drag_end(DragEvent) => DragEnd;
        /// Touchpad pinch; an extension the DOM lacks.
        on_pinch(PinchEvent) => Pinch;
        /// Input-method composition and committed text for the focused element (DOM
        /// `compositionupdate` / `beforeinput`); used by text fields.
        on_text_input(TextInputEvent) => TextInput;
        /// HTML `dragenter` and `dragover`: a drag from any app is over this element. Call
        /// `event.accept(mime_type)` to let it drop here.
        on_drag_over(DragOverEvent) => DragOver;
        /// HTML `dragleave`: the drag left this element without dropping.
        on_drag_leave(DragLeaveEvent) => DragLeave;
        /// HTML `drop`: data dropped on this element in the type a `dragover` handler accepted.
        on_drop(DropEvent) => Drop;
    }

    /// A keyboard shortcut that clicks this element while it is mounted.
    fn shortcut(self, keybind: Keybind) -> Self {
        self.modifier(Modifier::Shortcut(keybind))
    }
}

impl<T: Styled> Listeners for T {}
