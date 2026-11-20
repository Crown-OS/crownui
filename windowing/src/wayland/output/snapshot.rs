use sctk::{
    output::{OutputData, OutputInfo},
    reexports::client::{Proxy, protocol::wl_output::WlOutput},
};

use crate::wayland::{event_loop::WaylandPlatform, protocols::output_info::CrownOutput};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OutputId(pub(super) u32);

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
    /// How the CrownOS compositor treats the output; `None` on other compositors, or when the
    /// compositor did not describe it within a roundtrip.
    pub crown: Option<CrownOutput>,
}

impl Output {
    fn new(proxy: WlOutput, info: OutputInfo, crown: Option<CrownOutput>) -> Self {
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
            crown,
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
        let crown = self
            .crown
            .output_info
            .as_ref()
            .and_then(|global| global.described(OutputId(info.id)))
            .cloned();
        Some(Output::new(proxy, info, crown))
    }
}
