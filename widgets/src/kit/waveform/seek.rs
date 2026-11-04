use geometry::Size;
use input::{Key, NamedKey};
use reactive::{Memo, Signal, Store};
use view::{Cx, Listener, NodeId};

use super::metrics::KEY_STEP;
use crate::util::unit_interval::clamp_unit;

pub(super) type SeekHandler = Box<dyn FnMut(&mut Cx, f32)>;

/// Turns pointer positions and arrow keys on a waveform into `0..=1` seek positions.
#[derive(Clone, Copy)]
pub(super) struct Seek {
    pub(super) handler: Store<Option<SeekHandler>>,
    pub(super) size: Signal<Size>,
    pub(super) progress: Memo<f32>,
}

impl Seek {
    fn report(self, cx: &mut Cx, position: f32) {
        if let Some(mut handler) = self.handler.update(cx, Option::take) {
            handler(cx, clamp_unit(position));
            self.handler.update(cx, |slot| *slot = Some(handler));
        }
    }

    fn follow_pointer(self, cx: &mut Cx, x: f32) {
        let width = self.size.get(cx).width;
        if width > 0.0 {
            self.report(cx, x / width);
        }
    }

    fn after_key(self, cx: &mut Cx, key: NamedKey) -> Option<f32> {
        let progress = self.progress.get(cx);
        match key {
            NamedKey::ArrowRight | NamedKey::ArrowUp => Some(progress + KEY_STEP),
            NamedKey::ArrowLeft | NamedKey::ArrowDown => Some(progress - KEY_STEP),
            NamedKey::Home => Some(0.0),
            NamedKey::End => Some(1.0),
            _ => None,
        }
    }

    pub(super) fn attach(self, cx: &mut Cx, node: NodeId) {
        let listeners = [
            Listener::Click(Box::new(move |cx, event| {
                self.follow_pointer(cx, event.position.x)
            })),
            Listener::DragStart(Box::new(move |cx, event| {
                self.follow_pointer(cx, event.position.x)
            })),
            Listener::Drag(Box::new(move |cx, event| {
                self.follow_pointer(cx, event.position.x)
            })),
            Listener::KeyDown(Box::new(move |cx, event| {
                if let Key::Named(key) = event.key
                    && let Some(position) = self.after_key(cx, key)
                {
                    event.prevent_default();
                    self.report(cx, position);
                }
            })),
        ];
        for listener in listeners {
            cx.add_listener(node, listener);
        }
    }
}
