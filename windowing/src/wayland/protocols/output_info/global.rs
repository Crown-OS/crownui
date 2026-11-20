use std::collections::HashMap;

use crownos_protocols::output_info::v1::client::{
    crownos_output_info_manager_v1::CrownosOutputInfoManagerV1,
    crownos_output_info_v1::{self, CrownosOutputInfoV1},
};
use sctk::{
    dispatch2::Dispatch2,
    reexports::client::{
        Connection, QueueHandle,
        globals::GlobalList,
        protocol::{wl_callback, wl_callback::WlCallback, wl_output::WlOutput},
    },
};

use super::{
    Announcement, CrownOutput, OutputReport,
    info_event::{InfoBatch, apply_info_event},
};
use crate::wayland::{event_loop::WaylandPlatform, output::OutputId, util::no_events::NoEvents};

/// `crownos_output_info_manager_v1` and one `crownos_output_info_v1` for each output.
pub(crate) struct OutputInfoGlobal {
    manager: CrownosOutputInfoManagerV1,
    outputs: HashMap<OutputId, ObservedOutput>,
}

struct ObservedOutput {
    output: WlOutput,
    info: CrownosOutputInfoV1,
    received: CrownOutput,
    described: Option<CrownOutput>,
    announcement: Announcement,
}

impl OutputInfoGlobal {
    pub(crate) fn bind(globals: &GlobalList, qh: &QueueHandle<WaylandPlatform>) -> Option<Self> {
        Some(Self {
            manager: globals.bind(qh, 1..=1, NoEvents).ok()?,
            outputs: HashMap::new(),
        })
    }

    pub(crate) fn described(&self, output: OutputId) -> Option<&CrownOutput> {
        self.outputs.get(&output)?.described.as_ref()
    }

    pub(crate) fn output_updated(&self, output: OutputId) -> OutputReport {
        self.outputs
            .get(&output)
            .map_or(OutputReport::Changed, |observed| {
                observed.announcement.output_updated()
            })
    }

    /// Stops observing `output`; returns whether the app was told about it.
    pub(crate) fn forget(&mut self, output: OutputId) -> bool {
        self.outputs.remove(&output).is_none_or(|observed| {
            observed.info.destroy();
            observed.announcement.reports_removal()
        })
    }
}

impl WaylandPlatform {
    /// Asks the compositor how it treats `output`; returns what to tell the app right away.
    pub(crate) fn observe_output_info(&mut self, output: &WlOutput) -> OutputReport {
        let (Some(global), Some(id)) = (self.crown.output_info.as_mut(), OutputId::of(output))
        else {
            return OutputReport::Added;
        };
        let qh = &self.queue_handle;
        let info = global
            .manager
            .get_output(output, qh, OutputInfoData { output: id });
        self.connection
            .display()
            .sync(qh, OutputInfoDeadline { output: id });
        global.outputs.insert(
            id,
            ObservedOutput {
                output: output.clone(),
                info,
                received: CrownOutput::default(),
                described: None,
                announcement: Announcement::default(),
            },
        );
        OutputReport::Silent
    }

    fn update_observed_output(
        &mut self,
        id: OutputId,
        update: impl FnOnce(&mut ObservedOutput) -> OutputReport,
    ) {
        let Some(observed) = self
            .crown
            .output_info
            .as_mut()
            .and_then(|global| global.outputs.get_mut(&id))
        else {
            return;
        };
        let report = update(observed);
        let output = observed.output.clone();
        self.report_output(output, report);
    }
}

pub(crate) struct OutputInfoData {
    output: OutputId,
}

impl Dispatch2<CrownosOutputInfoV1, WaylandPlatform> for OutputInfoData {
    fn event(
        &self,
        state: &mut WaylandPlatform,
        _: &CrownosOutputInfoV1,
        event: crownos_output_info_v1::Event,
        _: &Connection,
        _: &QueueHandle<WaylandPlatform>,
    ) {
        state.update_observed_output(self.output, |observed| {
            if apply_info_event(&mut observed.received, event) == InfoBatch::Pending {
                return OutputReport::Silent;
            }
            observed.described = Some(observed.received.clone());
            observed.announcement.info_done()
        });
    }
}

pub(crate) struct OutputInfoDeadline {
    output: OutputId,
}

impl Dispatch2<WlCallback, WaylandPlatform> for OutputInfoDeadline {
    fn event(
        &self,
        state: &mut WaylandPlatform,
        _: &WlCallback,
        event: wl_callback::Event,
        _: &Connection,
        _: &QueueHandle<WaylandPlatform>,
    ) {
        if let wl_callback::Event::Done { .. } = event {
            state.update_observed_output(self.output, |observed| {
                observed.announcement.deadline_passed()
            });
        }
    }
}
