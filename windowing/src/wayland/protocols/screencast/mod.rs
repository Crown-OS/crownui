mod events;

use std::os::fd::BorrowedFd;

use crownos_protocols::screencast::v1::client::{
    crownos_screencast_manager_v1::CrownosScreencastManagerV1,
    crownos_screencast_session_v1::CrownosScreencastSessionV1,
};
pub use crownos_protocols::screencast::v1::client::{
    crownos_screencast_manager_v1::{Capability as ScreencastCapabilities, CursorMode},
    crownos_screencast_session_v1::StopReason,
};
pub use events::{
    BufferConstraints, CursorImage, DamageRect, DmabufFormat, ScreencastEvent, ScreencastFrame,
};
use sctk::reexports::client::{
    Proxy, QueueHandle, backend::ObjectId, globals::GlobalList, protocol::wl_buffer::WlBuffer,
};

use crate::wayland::{
    error::Error, event_loop::WaylandPlatform, output::Output, util::u64_halves::split_u64,
};
pub(crate) use events::PendingScreencast;
use events::{ScreencastManagerData, ScreencastSessionData};

pub(crate) struct ScreencastGlobal {
    manager: CrownosScreencastManagerV1,
    capabilities: ScreencastCapabilities,
}

impl ScreencastGlobal {
    pub(crate) fn bind(globals: &GlobalList, qh: &QueueHandle<WaylandPlatform>) -> Option<Self> {
        Some(Self {
            manager: globals.bind(qh, 1..=1, ScreencastManagerData).ok()?,
            capabilities: ScreencastCapabilities::empty(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScreencastId(ObjectId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SessionPhase {
    Configuring,
    ConfiguringWithTimelines,
    Started,
}

/// A capture of one output into a ring of client-owned dmabufs. Dropping it ends the session.
#[derive(Debug)]
pub struct ScreencastSession {
    proxy: CrownosScreencastSessionV1,
    phase: SessionPhase,
    explicit_sync_supported: bool,
}

impl WaylandPlatform {
    pub fn screencast_capabilities(&self) -> ScreencastCapabilities {
        self.crown
            .screencast
            .as_ref()
            .map_or(ScreencastCapabilities::empty(), |global| {
                global.capabilities
            })
    }

    /// The first [`ScreencastEvent::Constraints`] arrives right after this.
    pub fn create_screencast_session(
        &mut self,
        output: &Output,
        cursor_mode: CursorMode,
    ) -> Result<ScreencastSession, Error> {
        let global = self
            .crown
            .screencast
            .as_ref()
            .ok_or(Error::MissingGlobal("crownos_screencast_manager_v1"))?;
        let proxy = global.manager.create_output_session(
            &output.proxy,
            cursor_mode,
            &self.queue_handle,
            ScreencastSessionData,
        );
        let connection = &self.connection;
        self.screencast_batches
            .retain(|id, _| connection.object_info(id.clone()).is_ok());
        Ok(ScreencastSession {
            proxy,
            phase: SessionPhase::Configuring,
            explicit_sync_supported: global
                .capabilities
                .contains(ScreencastCapabilities::ExplicitSync),
        })
    }
}

impl ScreencastSession {
    pub const MAX_SLOTS: u32 = 4;

    pub fn id(&self) -> ScreencastId {
        ScreencastId(self.proxy.id())
    }

    /// Attach each slot once per constraints sequence; the buffer must satisfy those
    /// constraints and outlive its attachment.
    pub fn attach_buffer(&self, slot: u32, buffer: &WlBuffer) -> Result<(), Error> {
        self.proxy.attach_buffer(Self::checked(slot)?, buffer);
        Ok(())
    }

    /// Switches the session to explicit sync: frames are signalled on `acquire`,
    /// releases are signalled by the client on `release`.
    pub fn set_timelines(
        &mut self,
        acquire: BorrowedFd<'_>,
        release: BorrowedFd<'_>,
    ) -> Result<(), Error> {
        if !self.explicit_sync_supported {
            return Err(Error::ExplicitSyncUnsupported);
        }
        if self.phase != SessionPhase::Configuring {
            return Err(Error::ScreencastTimelinesRejected);
        }
        self.proxy.set_timelines(acquire, release);
        self.phase = SessionPhase::ConfiguringWithTimelines;
        Ok(())
    }

    /// `max_fps_mhz` caps the frame rate in millihertz; zero means the source's own refresh.
    pub fn start(&mut self, max_fps_mhz: u32) -> Result<(), Error> {
        if self.phase == SessionPhase::Started {
            return Err(Error::ScreencastAlreadyStarted);
        }
        self.proxy.start(max_fps_mhz);
        self.phase = SessionPhase::Started;
        Ok(())
    }

    /// Hands a slot back. With timelines, `release_point` must exceed every earlier one;
    /// without them it is ignored.
    pub fn release(&self, slot: u32, release_point: u64) -> Result<(), Error> {
        let (high, low) = split_u64(release_point);
        self.proxy.release(Self::checked(slot)?, high, low);
        Ok(())
    }

    /// Asks for one complete frame, e.g. to answer a decoder's keyframe request.
    pub fn force_frame(&self) {
        self.proxy.force_frame();
    }

    pub fn stop(&self) {
        self.proxy.stop();
    }

    fn checked(slot: u32) -> Result<u32, Error> {
        if slot < Self::MAX_SLOTS {
            Ok(slot)
        } else {
            Err(Error::InvalidScreencastSlot(slot))
        }
    }
}

impl Drop for ScreencastSession {
    fn drop(&mut self) {
        self.proxy.destroy();
    }
}
