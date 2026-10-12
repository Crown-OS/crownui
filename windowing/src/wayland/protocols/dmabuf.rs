use std::os::fd::BorrowedFd;

use sctk::{
    dmabuf::{DmabufFeedback, DmabufHandler, DmabufState},
    reexports::{
        client::{Connection, QueueHandle, protocol::wl_buffer::WlBuffer},
        protocols::wp::linux_dmabuf::zv1::client::{
            zwp_linux_buffer_params_v1::{Flags, ZwpLinuxBufferParamsV1},
            zwp_linux_dmabuf_feedback_v1::ZwpLinuxDmabufFeedbackV1,
        },
    },
};

use crate::wayland::{error::Error, event_loop::WaylandPlatform};

#[derive(Debug, Clone, Copy)]
pub struct DmabufPlane<'a> {
    pub fd: BorrowedFd<'a>,
    pub offset: u32,
    pub stride: u32,
}

impl WaylandPlatform {
    /// Wraps GPU memory, e.g. a GBM or Vulkan allocation, in a `wl_buffer` for a screencast ring
    /// or a surface. The compositor kills the connection if it cannot import the buffer.
    pub fn create_dmabuf_buffer(
        &self,
        planes: &[DmabufPlane<'_>],
        width: i32,
        height: i32,
        fourcc: u32,
        modifier: u64,
    ) -> Result<WlBuffer, Error> {
        let params = self
            .dmabuf_state
            .create_params(&self.queue_handle)
            .map_err(|_| Error::MissingGlobal("zwp_linux_dmabuf_v1"))?;
        for (plane_index, plane) in (0..).zip(planes) {
            params.add(plane.fd, plane_index, plane.offset, plane.stride, modifier);
        }
        let (buffer, params) =
            params.create_immed(width, height, fourcc, Flags::empty(), &self.queue_handle);
        params.destroy();
        Ok(buffer)
    }
}

impl DmabufHandler for WaylandPlatform {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self.dmabuf_state
    }

    fn dmabuf_feedback(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &ZwpLinuxDmabufFeedbackV1,
        feedback: DmabufFeedback,
    ) {
        self.main_device = Some(feedback.main_device());
    }

    fn created(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &ZwpLinuxBufferParamsV1,
        _: WlBuffer,
    ) {
    }

    fn failed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &ZwpLinuxBufferParamsV1) {}

    fn released(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlBuffer) {}
}
