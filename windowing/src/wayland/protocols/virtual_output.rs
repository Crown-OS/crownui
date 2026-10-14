use std::ops::RangeInclusive;

pub use crownos_protocols::virtual_output::v1::client::crownos_virtual_output_v1::CloseReason;
use crownos_protocols::virtual_output::v1::client::crownos_virtual_output_v1::{
    self, CrownosVirtualOutputV1,
};
use sctk::{
    dispatch2::Dispatch2,
    reexports::client::{Connection, Proxy, QueueHandle, backend::ObjectId},
};

use super::CrownEvent;
use crate::dpi::ScaleFactor;
use crate::wayland::{
    error::Error,
    event_loop::{QueuedEvent, WaylandPlatform},
};

const NAME_LENGTH: RangeInclusive<usize> = 1..=32;
const MODE_LENGTH: RangeInclusive<u32> = 1..=16384;
const SCALE_120THS: RangeInclusive<u32> = 60..=480;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VirtualOutputMode {
    pub width: u32,
    pub height: u32,
    /// Nominal refresh in millihertz; should match the rate frames will be requested at.
    pub refresh_mhz: u32,
    pub scale: ScaleFactor,
}

impl VirtualOutputMode {
    fn validate(self) -> Result<Self, Error> {
        let in_range = MODE_LENGTH.contains(&self.width)
            && MODE_LENGTH.contains(&self.height)
            && self.refresh_mhz != 0
            && SCALE_120THS.contains(&self.scale.as_120ths());
        in_range
            .then_some(self)
            .ok_or(Error::InvalidVirtualOutputMode)
    }
}

fn validate_name(name: &str) -> Result<&str, Error> {
    let valid = NAME_LENGTH.contains(&name.len())
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-');
    valid
        .then_some(name)
        .ok_or_else(|| Error::InvalidVirtualOutputName(name.to_owned()))
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VirtualOutputId(ObjectId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VirtualOutputEvent {
    /// The output exists; `name` is the `wl_output` name to look up with
    /// [`WaylandPlatform::output_by_name`] once its [`OutputEvent::Added`](crate::wayland::OutputEvent::Added) arrives.
    Created { name: String },
    /// The output is gone; drop the handle.
    Closed(CloseReason),
}

/// An output with no display behind it, paced by [`VirtualOutput::request_frame`].
/// Dropping it unplugs the output.
#[derive(Debug)]
pub struct VirtualOutput {
    proxy: CrownosVirtualOutputV1,
}

impl WaylandPlatform {
    pub fn create_virtual_output(
        &self,
        name: &str,
        mode: VirtualOutputMode,
    ) -> Result<VirtualOutput, Error> {
        let manager = self
            .crown
            .virtual_outputs
            .as_ref()
            .ok_or(Error::MissingGlobal("crownos_virtual_output_manager_v1"))?;
        let name = validate_name(name)?;
        let mode = mode.validate()?;
        let proxy = manager.create_output(
            name.to_owned(),
            mode.width as i32,
            mode.height as i32,
            mode.refresh_mhz,
            mode.scale.as_120ths(),
            &self.queue_handle,
            VirtualOutputData,
        );
        Ok(VirtualOutput { proxy })
    }
}

impl VirtualOutput {
    pub fn id(&self) -> VirtualOutputId {
        VirtualOutputId(self.proxy.id())
    }

    pub fn set_mode(&self, mode: VirtualOutputMode) -> Result<(), Error> {
        let mode = mode.validate()?;
        self.proxy.set_mode(
            mode.width as i32,
            mode.height as i32,
            mode.refresh_mhz,
            mode.scale.as_120ths(),
        );
        Ok(())
    }

    /// Advances the output's frame clock by one frame.
    pub fn request_frame(&self) {
        self.proxy.request_frame();
    }
}

impl Drop for VirtualOutput {
    fn drop(&mut self) {
        self.proxy.destroy();
    }
}

pub(crate) struct VirtualOutputData;

impl Dispatch2<CrownosVirtualOutputV1, WaylandPlatform> for VirtualOutputData {
    fn event(
        &self,
        state: &mut WaylandPlatform,
        output: &CrownosVirtualOutputV1,
        event: crownos_virtual_output_v1::Event,
        _: &Connection,
        _: &QueueHandle<WaylandPlatform>,
    ) {
        let event = match event {
            crownos_virtual_output_v1::Event::Created { name } => {
                VirtualOutputEvent::Created { name }
            }
            crownos_virtual_output_v1::Event::Closed { reason } => {
                VirtualOutputEvent::Closed(reason.into_result().unwrap_or(CloseReason::Failed))
            }
            _ => return,
        };
        state
            .pending_events
            .push(QueuedEvent::Crown(CrownEvent::VirtualOutput {
                output: VirtualOutputId(output.id()),
                event,
            }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mode(width: u32, scale_120ths: u32) -> VirtualOutputMode {
        VirtualOutputMode {
            width,
            height: 1080,
            refresh_mhz: 60_000,
            scale: ScaleFactor::from_120ths(scale_120ths),
        }
    }

    #[test]
    fn names_outside_the_protocol_alphabet_are_rejected() {
        assert!(validate_name("tablet-2_screen").is_ok());
        assert!(validate_name("").is_err());
        assert!(validate_name("has space").is_err());
        assert!(validate_name(&"x".repeat(33)).is_err());
    }

    #[test]
    fn modes_outside_the_protocol_ranges_are_rejected() {
        assert!(mode(1920, 120).validate().is_ok());
        assert!(mode(0, 120).validate().is_err());
        assert!(mode(16385, 120).validate().is_err());
        assert!(mode(1920, 59).validate().is_err());
        assert!(mode(1920, 481).validate().is_err());
    }
}
