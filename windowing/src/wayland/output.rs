use sctk::{
    output::{OutputData, OutputHandler, OutputInfo, OutputState},
    reexports::client::{Connection, Proxy, QueueHandle, protocol::wl_output::WlOutput},
};

use crate::wayland::event_loop::{QueuedEvent, WaylandPlatform};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OutputId(u32);

impl OutputId {
    pub(crate) fn of(output: &WlOutput) -> Option<Self> {
        output
            .data::<OutputData>()
            .map(|data| data.with_output_info(|info| Self(info.id)))
    }
}

/// A snapshot of a monitor, physical or virtual.
#[derive(Debug, Clone)]
pub struct Output {
    pub(crate) proxy: WlOutput,
    pub id: OutputId,
    pub name: Option<String>,
    pub description: Option<String>,
    pub logical_position: Option<(i32, i32)>,
    pub logical_size: Option<(i32, i32)>,
    pub scale_factor: i32,
    pub refresh_mhz: Option<i32>,
}

impl Output {
    fn new(proxy: WlOutput, info: OutputInfo) -> Self {
        Self {
            proxy,
            id: OutputId(info.id),
            refresh_mhz: info
                .modes
                .iter()
                .find(|mode| mode.current)
                .map(|mode| mode.refresh_rate),
            name: info.name,
            description: info.description,
            logical_position: info.logical_position,
            logical_size: info.logical_size,
            scale_factor: info.scale_factor,
        }
    }
}

#[derive(Debug, Clone)]
pub enum OutputEvent {
    Added(Output),
    Changed(Output),
    Removed(OutputId),
}

impl WaylandPlatform {
    pub fn outputs(&self) -> impl Iterator<Item = Output> + '_ {
        self.output_state
            .outputs()
            .filter_map(|proxy| self.output(proxy))
    }

    /// Finds an output by its `wl_output` name, such as the one a virtual output reports when created.
    pub fn output_by_name(&self, name: &str) -> Option<Output> {
        self.outputs()
            .find(|output| output.name.as_deref() == Some(name))
    }

    pub(crate) fn output(&self, proxy: WlOutput) -> Option<Output> {
        let info = self.output_state.info(&proxy)?;
        Some(Output::new(proxy, info))
    }
}

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
