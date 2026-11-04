use crate::{SourceId, SourceInfo};

/// crown-voiced's enhanced nodes are Audio/Source nodes too; apps reach them through
/// [`Enhancement`](crate::Enhancement), never by picking them.
const VOICE_NODE_PREFIX: &str = "crownos-voice.";

/// A change to the PipeWire graph that matters to the source list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum GraphChange {
    SourceAdded {
        global: u32,
        id: SourceId,
        description: String,
    },
    Removed(u32),
    DefaultSource(Option<String>),
}

/// The Audio/Source nodes in the graph, in the order they appeared, and the default one.
#[derive(Debug, Default)]
pub(super) struct SourceGraph {
    sources: Vec<(u32, SourceInfo)>,
    default: Option<String>,
}

impl SourceGraph {
    /// Applies `change`, returning whether the published list changes.
    pub(super) fn apply(&mut self, change: GraphChange) -> bool {
        match change {
            GraphChange::SourceAdded {
                global,
                id,
                description,
            } => {
                if id.as_str().starts_with(VOICE_NODE_PREFIX) {
                    return false;
                }
                self.sources.retain(|(known, _)| *known != global);
                self.sources.push((
                    global,
                    SourceInfo {
                        id,
                        description,
                        is_default: false,
                    },
                ));
                true
            }
            GraphChange::Removed(global) => {
                let before = self.sources.len();
                self.sources.retain(|(known, _)| *known != global);
                self.sources.len() != before
            }
            GraphChange::DefaultSource(name) => {
                let changed = self.default != name;
                self.default = name;
                changed
            }
        }
    }

    pub(super) fn sources(&self) -> Vec<SourceInfo> {
        let default = self.default.as_deref();
        self.sources
            .iter()
            .map(|(_, source)| SourceInfo {
                is_default: Some(source.id.as_str()) == default,
                ..source.clone()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn added(global: u32, name: &str) -> GraphChange {
        GraphChange::SourceAdded {
            global,
            id: SourceId::new(name),
            description: format!("{name} description"),
        }
    }

    fn names(graph: &SourceGraph) -> Vec<(String, bool)> {
        graph
            .sources()
            .into_iter()
            .map(|source| (source.id.as_str().to_owned(), source.is_default))
            .collect()
    }

    #[test]
    fn sources_come_and_go_with_hotplug() {
        let mut graph = SourceGraph::default();
        assert!(graph.apply(added(40, "builtin")));
        assert!(graph.apply(added(57, "usb")));
        assert_eq!(
            names(&graph),
            [("builtin".into(), false), ("usb".into(), false)]
        );
        assert!(graph.apply(GraphChange::Removed(57)));
        assert!(!graph.apply(GraphChange::Removed(57)));
        assert_eq!(names(&graph), [("builtin".into(), false)]);
    }

    #[test]
    fn the_default_source_is_marked() {
        let mut graph = SourceGraph::default();
        graph.apply(added(40, "builtin"));
        graph.apply(added(57, "usb"));
        assert!(graph.apply(GraphChange::DefaultSource(Some("usb".into()))));
        assert!(!graph.apply(GraphChange::DefaultSource(Some("usb".into()))));
        assert_eq!(
            names(&graph),
            [("builtin".into(), false), ("usb".into(), true)]
        );
    }

    #[test]
    fn crown_voiced_nodes_are_hidden() {
        let mut graph = SourceGraph::default();
        assert!(!graph.apply(added(90, "crownos-voice.stream-1")));
        assert!(graph.sources().is_empty());
    }

    #[test]
    fn a_reannounced_global_replaces_the_old_entry() {
        let mut graph = SourceGraph::default();
        graph.apply(added(40, "builtin"));
        graph.apply(added(40, "renamed"));
        assert_eq!(names(&graph), [("renamed".into(), false)]);
    }
}
