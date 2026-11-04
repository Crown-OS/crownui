use std::{
    sync::{OnceLock, mpsc},
    thread,
};

use pipewire::{
    core::Core,
    loop_::{Loop, Timeout},
    metadata::{Metadata, MetadataListener},
    properties::PropertiesBox,
    registry::{GlobalObject, RegistryBox},
    spa::utils::dict::DictRef,
    types::ObjectType,
};
use tokio::sync::watch;

use crate::{
    AudioError, SourceId, SourceInfo,
    util::{default_node_name::default_node_name, pipewire_core::run_on_core},
};

use super::graph::{GraphChange, SourceGraph};

const THREAD_NAME: &str = "crownui-audio-sources";
const DEFAULT_SOURCE_KEY: &str = "default.audio.source";

static SOURCES: OnceLock<watch::Receiver<Vec<SourceInfo>>> = OnceLock::new();

/// The capture devices in the PipeWire graph, updated as they are plugged in and out and as
/// the default changes. One registry watcher thread serves the whole process; the list stays
/// empty when PipeWire is unavailable.
pub fn sources() -> watch::Receiver<Vec<SourceInfo>> {
    SOURCES.get_or_init(start_watcher).clone()
}

enum RegistryEvent {
    Graph(GraphChange),
    DefaultMetadata(GlobalObject<PropertiesBox>),
}

fn start_watcher() -> watch::Receiver<Vec<SourceInfo>> {
    let (published, sources) = watch::channel(Vec::new());
    let spawned = thread::Builder::new()
        .name(THREAD_NAME.into())
        .spawn(move || {
            if let Err(error) = run_on_core(|loop_, core| watch_registry(loop_, core, &published)) {
                tracing::warn!("audio sources are not watched: {error}");
            }
        });
    if let Err(error) = spawned {
        tracing::warn!("{}", AudioError::Thread(error));
    }
    sources
}

fn watch_registry(
    loop_: &Loop,
    core: &Core,
    published: &watch::Sender<Vec<SourceInfo>>,
) -> Result<(), AudioError> {
    let registry = core.get_registry().map_err(AudioError::pipewire)?;
    let (events, received) = mpsc::channel();
    let _registry_listener = registry
        .add_listener_local()
        .global({
            let events = events.clone();
            move |global| {
                if let Some(event) = registry_event(global) {
                    let _ = events.send(event);
                }
            }
        })
        .global_remove({
            let events = events.clone();
            move |global| {
                let _ = events.send(RegistryEvent::Graph(GraphChange::Removed(global)));
            }
        })
        .register();
    let mut graph = SourceGraph::default();
    let mut _default_metadata = None;
    loop {
        loop_.iterate(Timeout::Infinite);
        let mut changed = false;
        for event in received.try_iter() {
            match event {
                RegistryEvent::Graph(change) => changed |= graph.apply(change),
                RegistryEvent::DefaultMetadata(global) => {
                    _default_metadata = bind_default_metadata(&registry, &global, &events);
                }
            }
        }
        if changed {
            published.send_replace(graph.sources());
        }
    }
}

fn registry_event(global: &GlobalObject<&DictRef>) -> Option<RegistryEvent> {
    let property = |key: &str| global.props.and_then(|props| props.get(key));
    match global.type_ {
        ObjectType::Node if property("media.class")?.starts_with("Audio/Source") => {
            let name = property("node.name")?;
            let description = property("node.description")
                .or_else(|| property("node.nick"))
                .unwrap_or(name);
            Some(RegistryEvent::Graph(GraphChange::SourceAdded {
                global: global.id,
                id: SourceId::new(name),
                description: description.to_owned(),
            }))
        }
        ObjectType::Metadata if property("metadata.name") == Some("default") => {
            Some(RegistryEvent::DefaultMetadata(global.to_owned()))
        }
        _ => None,
    }
}

fn bind_default_metadata(
    registry: &RegistryBox<'_>,
    global: &GlobalObject<PropertiesBox>,
    events: &mpsc::Sender<RegistryEvent>,
) -> Option<(Metadata, MetadataListener)> {
    let metadata: Metadata = registry
        .bind(global)
        .inspect_err(|error| tracing::debug!("cannot bind the default metadata: {error}"))
        .ok()?;
    let events = events.clone();
    let listener = metadata
        .add_listener_local()
        .property(move |_, key, _, value| {
            if key.is_none_or(|key| key == DEFAULT_SOURCE_KEY) {
                let name = value.and_then(default_node_name).map(str::to_owned);
                let _ = events.send(RegistryEvent::Graph(GraphChange::DefaultSource(name)));
            }
            0
        })
        .register();
    Some((metadata, listener))
}
