use sctk::{
    output::{OutputHandler, OutputState},
    reexports::client::{Connection, QueueHandle, protocol::wl_output::WlOutput},
};

use super::{OutputEvent, OutputId};
use crate::wayland::{
    event_loop::{QueuedEvent, WaylandPlatform},
    protocols::output_info::OutputReport,
};

impl WaylandPlatform {
    pub(crate) fn report_output(&mut self, output: WlOutput, report: OutputReport) {
        let event = match (report, self.output(output)) {
            (OutputReport::Added, Some(output)) => OutputEvent::Added(output),
            (OutputReport::Changed, Some(output)) => OutputEvent::Changed(output),
            _ => return,
        };
        self.pending_events.push(QueuedEvent::Output(event));
    }
}

impl OutputHandler for WaylandPlatform {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, output: WlOutput) {
        let report = self.observe_output_info(&output);
        self.report_output(output, report);
        self.refit_layers();
    }

    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, output: WlOutput) {
        let report = match (&self.crown.output_info, OutputId::of(&output)) {
            (Some(global), Some(id)) => global.output_updated(id),
            _ => OutputReport::Changed,
        };
        self.report_output(output, report);
        self.refit_layers();
    }

    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, output: WlOutput) {
        let Some(id) = OutputId::of(&output) else {
            return;
        };
        let announced = self
            .crown
            .output_info
            .as_mut()
            .is_none_or(|global| global.forget(id));
        if announced {
            self.pending_events
                .push(QueuedEvent::Output(OutputEvent::Removed(id)));
        }
    }
}
