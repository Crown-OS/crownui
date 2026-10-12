use geometry::Size;
use input::{Key, NamedKey, WheelDelta};
use reactive::Signal;
use view::{Cx, Listener, NodeId};

use super::{
    metrics::{THUMB_HALF_WIDTH, WHEEL_NOTCH_SENSITIVITY, WHEEL_PIXEL_SENSITIVITY},
    scale::SliderScale,
};

pub(super) fn track_width(size: Size) -> f32 {
    (size.width - 2.0 * THUMB_HALF_WIDTH).max(0.0)
}

fn commit(cx: &mut Cx, value: Signal<f32>, next: f32) {
    if value.get(cx) != next {
        value.set(cx, next);
    }
}

pub(super) fn attach(
    cx: &mut Cx,
    node: NodeId,
    value: Signal<f32>,
    scale: SliderScale,
    size: Signal<Size>,
) {
    let follow_pointer = move |cx: &mut Cx, x: f32| {
        let width = track_width(size.get(cx));
        if width > 0.0 {
            commit(cx, value, scale.value_at((x - THUMB_HALF_WIDTH) / width));
        }
    };
    let listeners = [
        Listener::Click(Box::new(move |cx, event| {
            follow_pointer(cx, event.position.x)
        })),
        Listener::DragStart(Box::new(move |cx, event| {
            follow_pointer(cx, event.position.x)
        })),
        Listener::Drag(Box::new(move |cx, event| {
            follow_pointer(cx, event.position.x)
        })),
        Listener::KeyDown(Box::new(move |cx, event| match event.key {
            Key::Named(NamedKey::Enter | NamedKey::Space) => event.prevent_default(),
            Key::Named(key) => {
                let current = value.get(cx);
                if let Some(next) = scale.after_key(current, key, event.modifiers.shift_key) {
                    event.prevent_default();
                    commit(cx, value, next);
                }
            }
            _ => {}
        })),
        Listener::Wheel(Box::new(move |cx, event| {
            let steps = match event.delta {
                WheelDelta::Lines(lines) => lines_or_columns(lines) * WHEEL_NOTCH_SENSITIVITY,
                WheelDelta::Pixels(pixels) => lines_or_columns(pixels) * WHEEL_PIXEL_SENSITIVITY,
            };
            if steps != 0.0 {
                event.stop_propagation();
                let current = value.get(cx);
                commit(cx, value, scale.nudged(current, steps));
            }
        })),
    ];
    for listener in listeners {
        cx.add_listener(node, listener);
    }
}

fn lines_or_columns(delta: geometry::Vector) -> f32 {
    if delta.x != 0.0 { delta.x } else { delta.y }
}
