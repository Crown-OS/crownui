use input::{KeyboardEvent, PointerEvent, TextInputEvent, WheelEvent};

use crate::{
    Cx,
    event::{
        ClickEvent, DragEvent, DragLeaveEvent, DragOverEvent, DropEvent, Event, FocusEvent, InputEvent, LongPressEvent,
        PinchEvent, ScrollEvent,
    },
};

/// A boxed event handler.
pub type Handler<T> = Box<dyn FnMut(&mut Cx, &mut Event<T>)>;

/// Every event a node can listen to, named after DOM events.
#[allow(missing_docs)]
pub enum Listener {
    Click(Handler<ClickEvent>),
    DoubleClick(Handler<ClickEvent>),
    ContextMenu(Handler<ClickEvent>),
    LongPress(Handler<LongPressEvent>),
    PointerDown(Handler<PointerEvent>),
    PointerUp(Handler<PointerEvent>),
    PointerMove(Handler<PointerEvent>),
    PointerEnter(Handler<PointerEvent>),
    PointerLeave(Handler<PointerEvent>),
    PointerCancel(Handler<PointerEvent>),
    Wheel(Handler<WheelEvent>),
    Scroll(Handler<ScrollEvent>),
    KeyDown(Handler<KeyboardEvent>),
    KeyUp(Handler<KeyboardEvent>),
    Input(Handler<InputEvent>),
    Change(Handler<InputEvent>),
    Submit(Handler<InputEvent>),
    Focus(Handler<FocusEvent>),
    Blur(Handler<FocusEvent>),
    DragStart(Handler<DragEvent>),
    Drag(Handler<DragEvent>),
    DragEnd(Handler<DragEvent>),
    Pinch(Handler<PinchEvent>),
    TextInput(Handler<TextInputEvent>),
    DragOver(Handler<DragOverEvent>),
    DragLeave(Handler<DragLeaveEvent>),
    Drop(Handler<DropEvent>),
}

/// Which listener an event is for; used to look listeners up without matching closures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum ListenerKind {
    Click,
    DoubleClick,
    ContextMenu,
    LongPress,
    PointerDown,
    PointerUp,
    PointerMove,
    PointerEnter,
    PointerLeave,
    PointerCancel,
    Wheel,
    Scroll,
    KeyDown,
    KeyUp,
    Input,
    Change,
    Submit,
    Focus,
    Blur,
    DragStart,
    Drag,
    DragEnd,
    Pinch,
    TextInput,
    DragOver,
    DragLeave,
    Drop,
}

impl Listener {
    /// The kind of event this listener handles.
    pub fn kind(&self) -> ListenerKind {
        match self {
            Self::Click(_) => ListenerKind::Click,
            Self::DoubleClick(_) => ListenerKind::DoubleClick,
            Self::ContextMenu(_) => ListenerKind::ContextMenu,
            Self::LongPress(_) => ListenerKind::LongPress,
            Self::PointerDown(_) => ListenerKind::PointerDown,
            Self::PointerUp(_) => ListenerKind::PointerUp,
            Self::PointerMove(_) => ListenerKind::PointerMove,
            Self::PointerEnter(_) => ListenerKind::PointerEnter,
            Self::PointerLeave(_) => ListenerKind::PointerLeave,
            Self::PointerCancel(_) => ListenerKind::PointerCancel,
            Self::Wheel(_) => ListenerKind::Wheel,
            Self::Scroll(_) => ListenerKind::Scroll,
            Self::KeyDown(_) => ListenerKind::KeyDown,
            Self::KeyUp(_) => ListenerKind::KeyUp,
            Self::Input(_) => ListenerKind::Input,
            Self::Change(_) => ListenerKind::Change,
            Self::Submit(_) => ListenerKind::Submit,
            Self::Focus(_) => ListenerKind::Focus,
            Self::Blur(_) => ListenerKind::Blur,
            Self::DragStart(_) => ListenerKind::DragStart,
            Self::Drag(_) => ListenerKind::Drag,
            Self::DragEnd(_) => ListenerKind::DragEnd,
            Self::Pinch(_) => ListenerKind::Pinch,
            Self::TextInput(_) => ListenerKind::TextInput,
            Self::DragOver(_) => ListenerKind::DragOver,
            Self::DragLeave(_) => ListenerKind::DragLeave,
            Self::Drop(_) => ListenerKind::Drop,
        }
    }
}

/// An event payload together with the listener kind it targets.
pub(crate) enum Dispatchable {
    Click(ListenerKind, Event<ClickEvent>),
    LongPress(Event<LongPressEvent>),
    Pointer(ListenerKind, Event<PointerEvent>),
    Wheel(Event<WheelEvent>),
    Scroll(Event<ScrollEvent>),
    Key(ListenerKind, Event<KeyboardEvent>),
    Input(ListenerKind, Event<InputEvent>),
    Focus(ListenerKind, Event<FocusEvent>),
    Drag(ListenerKind, Event<DragEvent>),
    Pinch(Event<PinchEvent>),
    TextInput(Event<TextInputEvent>),
    DragOver(Event<DragOverEvent>),
    DragLeave(Event<DragLeaveEvent>),
    Drop(Event<DropEvent>),
}

impl Dispatchable {
    pub(crate) fn kind(&self) -> ListenerKind {
        match self {
            Self::Click(kind, _)
            | Self::Pointer(kind, _)
            | Self::Key(kind, _)
            | Self::Input(kind, _)
            | Self::Focus(kind, _)
            | Self::Drag(kind, _) => *kind,
            Self::LongPress(_) => ListenerKind::LongPress,
            Self::Wheel(_) => ListenerKind::Wheel,
            Self::Scroll(_) => ListenerKind::Scroll,
            Self::Pinch(_) => ListenerKind::Pinch,
            Self::TextInput(_) => ListenerKind::TextInput,
            Self::DragOver(_) => ListenerKind::DragOver,
            Self::DragLeave(_) => ListenerKind::DragLeave,
            Self::Drop(_) => ListenerKind::Drop,
        }
    }

    pub(crate) fn is_propagation_stopped(&self) -> bool {
        match self {
            Self::Click(_, event) => event.is_propagation_stopped(),
            Self::LongPress(event) => event.is_propagation_stopped(),
            Self::Pointer(_, event) => event.is_propagation_stopped(),
            Self::Wheel(event) => event.is_propagation_stopped(),
            Self::Scroll(event) => event.is_propagation_stopped(),
            Self::Key(_, event) => event.is_propagation_stopped(),
            Self::Input(_, event) => event.is_propagation_stopped(),
            Self::Focus(_, event) => event.is_propagation_stopped(),
            Self::Drag(_, event) => event.is_propagation_stopped(),
            Self::Pinch(event) => event.is_propagation_stopped(),
            Self::TextInput(event) => event.is_propagation_stopped(),
            Self::DragOver(event) => event.is_propagation_stopped(),
            Self::DragLeave(event) => event.is_propagation_stopped(),
            Self::Drop(event) => event.is_propagation_stopped(),
        }
    }

    pub(crate) fn is_default_prevented(&self) -> bool {
        match self {
            Self::Click(_, event) => event.is_default_prevented(),
            Self::LongPress(event) => event.is_default_prevented(),
            Self::Pointer(_, event) => event.is_default_prevented(),
            Self::Wheel(event) => event.is_default_prevented(),
            Self::Scroll(event) => event.is_default_prevented(),
            Self::Key(_, event) => event.is_default_prevented(),
            Self::Input(_, event) => event.is_default_prevented(),
            Self::Focus(_, event) => event.is_default_prevented(),
            Self::Drag(_, event) => event.is_default_prevented(),
            Self::Pinch(event) => event.is_default_prevented(),
            Self::TextInput(event) => event.is_default_prevented(),
            Self::DragOver(event) => event.is_default_prevented(),
            Self::DragLeave(event) => event.is_default_prevented(),
            Self::Drop(event) => event.is_default_prevented(),
        }
    }

    /// Calls `listener` if it handles this event; returns whether it did.
    pub(crate) fn invoke(&mut self, listener: &mut Listener, cx: &mut Cx) -> bool {
        match (self, listener) {
            (Self::Click(ListenerKind::Click, event), Listener::Click(handler))
            | (Self::Click(ListenerKind::DoubleClick, event), Listener::DoubleClick(handler))
            | (Self::Click(ListenerKind::ContextMenu, event), Listener::ContextMenu(handler)) => handler(cx, event),
            (Self::LongPress(event), Listener::LongPress(handler)) => handler(cx, event),
            (Self::Pointer(ListenerKind::PointerDown, event), Listener::PointerDown(handler))
            | (Self::Pointer(ListenerKind::PointerUp, event), Listener::PointerUp(handler))
            | (Self::Pointer(ListenerKind::PointerMove, event), Listener::PointerMove(handler))
            | (Self::Pointer(ListenerKind::PointerEnter, event), Listener::PointerEnter(handler))
            | (Self::Pointer(ListenerKind::PointerLeave, event), Listener::PointerLeave(handler))
            | (Self::Pointer(ListenerKind::PointerCancel, event), Listener::PointerCancel(handler)) => handler(cx, event),
            (Self::Wheel(event), Listener::Wheel(handler)) => handler(cx, event),
            (Self::Scroll(event), Listener::Scroll(handler)) => handler(cx, event),
            (Self::Key(ListenerKind::KeyDown, event), Listener::KeyDown(handler))
            | (Self::Key(ListenerKind::KeyUp, event), Listener::KeyUp(handler)) => handler(cx, event),
            (Self::Input(ListenerKind::Input, event), Listener::Input(handler))
            | (Self::Input(ListenerKind::Change, event), Listener::Change(handler))
            | (Self::Input(ListenerKind::Submit, event), Listener::Submit(handler)) => handler(cx, event),
            (Self::Focus(ListenerKind::Focus, event), Listener::Focus(handler))
            | (Self::Focus(ListenerKind::Blur, event), Listener::Blur(handler)) => handler(cx, event),
            (Self::Drag(ListenerKind::DragStart, event), Listener::DragStart(handler))
            | (Self::Drag(ListenerKind::Drag, event), Listener::Drag(handler))
            | (Self::Drag(ListenerKind::DragEnd, event), Listener::DragEnd(handler)) => handler(cx, event),
            (Self::Pinch(event), Listener::Pinch(handler)) => handler(cx, event),
            (Self::TextInput(event), Listener::TextInput(handler)) => handler(cx, event),
            (Self::DragOver(event), Listener::DragOver(handler)) => handler(cx, event),
            (Self::DragLeave(event), Listener::DragLeave(handler)) => handler(cx, event),
            (Self::Drop(event), Listener::Drop(handler)) => handler(cx, event),
            _ => return false,
        }
        true
    }
}
