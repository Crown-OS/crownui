use input::{KeyboardEvent, PhysicalKey};
use sctk::{
    reexports::client::{
        Connection, QueueHandle,
        protocol::{wl_keyboard::WlKeyboard, wl_surface::WlSurface},
    },
    seat::keyboard::{KeyEvent as SctkKeyEvent, KeyboardHandler, Keysym, Modifiers, RawModifiers},
};

use super::keymap::{key_from_keysym, modifiers_from_sctk};
use crate::{
    WindowEvent,
    wayland::{event_loop::WaylandPlatform, window::surface_window_id},
};

impl WaylandPlatform {
    pub(crate) fn keyboard_event(&self, event: SctkKeyEvent, repeat: bool) -> KeyboardEvent {
        KeyboardEvent {
            key: key_from_keysym(event.keysym),
            code: PhysicalKey(event.raw_code),
            text: event.utf8,
            repeat,
            modifiers: self.modifiers,
        }
    }

    pub(crate) fn queue_key_event(&mut self, event: WindowEvent) {
        if let Some(window_id) = self.keyboard_focus {
            self.push_window_event(window_id, event);
        }
    }
}

impl KeyboardHandler for WaylandPlatform {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlKeyboard,
        surface: &WlSurface,
        serial: u32,
        _: &[u32],
        _: &[Keysym],
    ) {
        self.input_serial = serial;
        let window_id = surface_window_id(surface);
        if self.windows.contains_key(&window_id) {
            self.keyboard_focus = Some(window_id);
            self.push_window_event(window_id, WindowEvent::Focused(true));
        }
    }

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlKeyboard,
        surface: &WlSurface,
        _: u32,
    ) {
        let window_id = surface_window_id(surface);
        if self.keyboard_focus == Some(window_id) {
            self.keyboard_focus = None;
            self.push_window_event(window_id, WindowEvent::Focused(false));
        }
    }

    fn press_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlKeyboard,
        serial: u32,
        event: SctkKeyEvent,
    ) {
        self.input_serial = serial;
        let event = self.keyboard_event(event, false);
        self.queue_key_event(WindowEvent::KeyDown(event));
    }

    fn repeat_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlKeyboard,
        _: u32,
        event: SctkKeyEvent,
    ) {
        let event = self.keyboard_event(event, true);
        self.queue_key_event(WindowEvent::KeyDown(event));
    }

    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlKeyboard,
        _: u32,
        event: SctkKeyEvent,
    ) {
        let event = self.keyboard_event(event, false);
        self.queue_key_event(WindowEvent::KeyUp(event));
    }

    fn update_modifiers(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlKeyboard,
        _: u32,
        modifiers: Modifiers,
        _: RawModifiers,
        _: u32,
    ) {
        self.modifiers = modifiers_from_sctk(modifiers);
        let modifiers = self.modifiers;
        self.queue_key_event(WindowEvent::ModifiersChanged(modifiers));
    }
}
