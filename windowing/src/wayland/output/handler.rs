use sctk::{
    output::{OutputHandler, OutputState},
    reexports::client::{Connection, QueueHandle, protocol::wl_output::WlOutput},
};

use super::{OutputEvent, OutputId};
use crate::wayland::event_loop::{QueuedEvent, WaylandPlatform};

impl OutputHandler for WaylandPlatform {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, output: WlOutput) {
        if let Some(output) = self.output(output) {
            self.pending_events
                .push(QueuedEvent::Output(OutputEvent::Added(output)));
        }
        self.refit_layers();
    }

    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, output: WlOutput) {
        if let Some(output) = self.output(output) {
            self.pending_events
                .push(QueuedEvent::Output(OutputEvent::Changed(output)));
        }
        self.refit_layers();
    }

    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, output: WlOutput) {
        if let Some(id) = OutputId::of(&output) {
            self.pending_events
                .push(QueuedEvent::Output(OutputEvent::Removed(id)));
        }
    }
}
