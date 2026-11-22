use std::{
    sync::{OnceLock, mpsc},
    thread,
};

use pipewire::{
    core::Core,
    loop_::{Loop, Timeout},
    registry::GlobalObject,
    spa::utils::dict::DictRef,
    types::ObjectType,
};
use tokio::sync::watch;

use crate::{CameraError, CameraId, CameraInfo, util::pipewire_core::run_on_core};

use super::list::CameraList;

const THREAD_NAME: &str = "crownui-cameras";
const VIDEO_SOURCE: &str = "Video/Source";
const CAMERA_ROLE: &str = "Camera";

static CAMERAS: OnceLock<watch::Receiver<Vec<CameraInfo>>> = OnceLock::new();

/// The cameras in the PipeWire graph, updated as they are plugged in and out. One registry
/// watcher thread serves the whole process; the list stays empty when PipeWire is unavailable.
pub fn cameras() -> watch::Receiver<Vec<CameraInfo>> {
    CAMERAS.get_or_init(start_watcher).clone()
}

enum RegistryEvent {
    Added(u32, CameraInfo),
    Removed(u32),
}

fn start_watcher() -> watch::Receiver<Vec<CameraInfo>> {
    let (published, cameras) = watch::channel(Vec::new());
    let spawned = thread::Builder::new()
        .name(THREAD_NAME.into())
        .spawn(move || {
            if let Err(error) = run_on_core(|loop_, core| watch_registry(loop_, core, &published)) {
                tracing::warn!("cameras are not watched: {error}");
            }
        });
    if let Err(error) = spawned {
        tracing::warn!("{}", CameraError::Thread(error));
    }
    cameras
}

fn watch_registry(
    loop_: &Loop,
    core: &Core,
    published: &watch::Sender<Vec<CameraInfo>>,
) -> Result<(), CameraError> {
    let registry = core.get_registry().map_err(CameraError::pipewire)?;
    let (events, received) = mpsc::channel();
    let _listener = registry
        .add_listener_local()
        .global({
            let events = events.clone();
            move |global| {
                if let Some(camera) = camera_of(global) {
                    let _ = events.send(RegistryEvent::Added(global.id, camera));
                }
            }
        })
        .global_remove(move |global| {
            let _ = events.send(RegistryEvent::Removed(global));
        })
        .register();
    let mut list = CameraList::default();
    loop {
        loop_.iterate(Timeout::Infinite);
        let mut changed = false;
        for event in received.try_iter() {
            changed |= match event {
                RegistryEvent::Added(global, camera) => list.add(global, camera),
                RegistryEvent::Removed(global) => list.remove(global),
            };
        }
        if changed {
            published.send_replace(list.cameras());
        }
    }
}

fn camera_of(global: &GlobalObject<&DictRef>) -> Option<CameraInfo> {
    let property = |key: &str| global.props.and_then(|props| props.get(key));
    let is_camera = global.type_ == ObjectType::Node
        && property("media.class") == Some(VIDEO_SOURCE)
        && property("media.role") == Some(CAMERA_ROLE);
    if !is_camera {
        return None;
    }
    let name = property("node.name")?;
    let description = property("node.description")
        .or_else(|| property("node.nick"))
        .unwrap_or(name);
    Some(CameraInfo {
        id: CameraId::new(name),
        description: description.to_owned(),
    })
}
