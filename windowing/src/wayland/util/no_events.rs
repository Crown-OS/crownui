use sctk::{
    dispatch2::Dispatch2,
    reexports::client::{Connection, Proxy, QueueHandle},
};

use crate::wayland::event_loop::WaylandPlatform;

/// User data for objects whose interface defines no events.
pub(crate) struct NoEvents;

impl<I: Proxy> Dispatch2<I, WaylandPlatform> for NoEvents {
    fn event(
        &self,
        _: &mut WaylandPlatform,
        _: &I,
        _: I::Event,
        _: &Connection,
        _: &QueueHandle<WaylandPlatform>,
    ) {
    }
}
