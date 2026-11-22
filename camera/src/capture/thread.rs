use std::{
    sync::mpsc,
    thread::{self, JoinHandle},
};

use crossbeam_channel::{Receiver, Sender, TrySendError};
use pipewire::{
    self as pw,
    properties::properties,
    spa::{param::ParamType, pod::Pod, utils::Direction},
    stream::{Stream, StreamBox, StreamFlags, StreamState},
};

use crate::{CameraError, CameraOptions, CaptureFormat, util::monotonic};

use super::{CaptureEvent, EncodedFrame, pods};

const THREAD_NAME: &str = "crownui-camera";
const STREAM_NAME: &str = "crownui-camera";

/// A camera stream on its own PipeWire loop thread, stopped and joined when dropped.
pub(crate) struct CaptureThread {
    quit: pw::channel::Sender<()>,
    thread: Option<JoinHandle<()>>,
}

impl CaptureThread {
    /// Starts streaming as `options` describe; returns once PipeWire accepted the stream.
    /// Frames go to `events`, which drops them while full; their memory comes back on `recycled`.
    pub(crate) fn spawn(
        options: CameraOptions,
        events: Sender<CaptureEvent>,
        recycled: Receiver<Vec<u8>>,
    ) -> Result<Self, CameraError> {
        let (quit, quit_receiver) = pw::channel::channel();
        let (ready_sender, ready) = mpsc::sync_channel(1);
        let thread = thread::Builder::new()
            .name(THREAD_NAME.into())
            .spawn(move || {
                let delivery = Delivery {
                    format: None,
                    events: events.clone(),
                    recycled,
                };
                if let Err(error) = run(&options, delivery, quit_receiver, &ready_sender) {
                    let _ = events.send(CaptureEvent::Failed(error.to_string()));
                    let _ = ready_sender.send(Err(error));
                }
            })
            .map_err(CameraError::Thread)?;
        let capture = Self {
            quit,
            thread: Some(thread),
        };
        match ready.recv() {
            Ok(Ok(())) => Ok(capture),
            Ok(Err(error)) => Err(error),
            Err(_) => Err(CameraError::Stopped),
        }
    }
}

impl Drop for CaptureThread {
    fn drop(&mut self) {
        let _ = self.quit.send(());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// The stream's user data: copies each buffer out of the real-time loop.
struct Delivery {
    format: Option<CaptureFormat>,
    events: Sender<CaptureEvent>,
    recycled: Receiver<Vec<u8>>,
}

impl Delivery {
    fn deliver(&mut self, stream: &Stream) {
        let Some(format) = self.format else {
            return;
        };
        let Some(mut buffer) = stream.dequeue_buffer() else {
            return;
        };
        let Some(data) = buffer.datas_mut().first_mut() else {
            return;
        };
        let chunk = data.chunk();
        let (offset, size, stride) = (
            chunk.offset() as usize,
            chunk.size() as usize,
            chunk.stride().unsigned_abs(),
        );
        let Some(bytes) = data
            .data()
            .and_then(|bytes| bytes.get(offset..offset + size))
        else {
            return;
        };
        let mut owned = self.recycled.try_recv().unwrap_or_default();
        owned.clear();
        owned.extend_from_slice(bytes);
        let frame = EncodedFrame {
            format,
            stride,
            pts: monotonic::now(),
            bytes: owned,
        };
        if let Err(TrySendError::Disconnected(_)) = self.events.try_send(CaptureEvent::Frame(frame))
        {
            self.format = None;
        }
    }
}

fn run(
    options: &CameraOptions,
    delivery: Delivery,
    quit: pw::channel::Receiver<()>,
    ready: &mpsc::SyncSender<Result<(), CameraError>>,
) -> Result<(), CameraError> {
    pw::init();
    let main_loop = pw::main_loop::MainLoopRc::new(None).map_err(CameraError::pipewire)?;
    let context = pw::context::ContextRc::new(&main_loop, None).map_err(CameraError::pipewire)?;
    let core = context.connect_rc(None).map_err(CameraError::pipewire)?;
    let mut properties = properties! {
        *pw::keys::MEDIA_TYPE => "Video",
        *pw::keys::MEDIA_CATEGORY => "Capture",
        *pw::keys::MEDIA_ROLE => "Camera",
        *pw::keys::NODE_NAME => STREAM_NAME,
    };
    if let Some(camera) = &options.camera {
        properties.insert(*pw::keys::TARGET_OBJECT, camera.as_str());
        properties.insert(*pw::keys::NODE_DONT_RECONNECT, "true");
    }
    let stream = StreamBox::new(&core, STREAM_NAME, properties).map_err(CameraError::pipewire)?;
    let buffers = pods::cpu_buffers()?;
    let failures = delivery.events.clone();
    let _listener = stream
        .add_local_listener_with_user_data(delivery)
        .param_changed(move |stream, delivery, id, param| {
            if id != ParamType::Format.as_raw() {
                return;
            }
            delivery.format = param.and_then(pods::negotiated);
            if delivery.format.is_some()
                && let Some(buffers) = Pod::from_bytes(&buffers)
            {
                let _ = stream.update_params(&mut [buffers]);
            }
        })
        .state_changed({
            let main_loop = main_loop.clone();
            move |_, _, _, new| match new {
                StreamState::Error(message) => {
                    let _ = failures.send(CaptureEvent::Failed(message));
                    main_loop.quit();
                }
                StreamState::Unconnected => {
                    let _ = failures.send(CaptureEvent::Failed("the camera went away".into()));
                    main_loop.quit();
                }
                _ => {}
            }
        })
        .process(|stream, delivery| delivery.deliver(stream))
        .register()
        .map_err(CameraError::pipewire)?;
    let offers = pods::enum_formats(options)?;
    let mut params: Vec<&Pod> = offers
        .iter()
        .filter_map(|bytes| Pod::from_bytes(bytes))
        .collect();
    stream
        .connect(
            Direction::Input,
            None,
            StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS,
            &mut params,
        )
        .map_err(CameraError::pipewire)?;
    let _quit = quit.attach(main_loop.loop_(), {
        let main_loop = main_loop.clone();
        move |()| main_loop.quit()
    });
    let _ = ready.send(Ok(()));
    main_loop.run();
    Ok(())
}
