use pipewire::stream::StreamState as PipeWireState;

use crate::{StreamState, UnenhancedReason};

/// How the PipeWire capture stream is doing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CaptureStatus {
    Waiting,
    Streaming,
    Failed(String),
}

impl CaptureStatus {
    /// `None` for `Unconnected`, which only happens while a capture is being torn down.
    pub(super) fn from_pipewire(state: PipeWireState) -> Option<Self> {
        match state {
            PipeWireState::Streaming => Some(Self::Streaming),
            PipeWireState::Connecting | PipeWireState::Paused => Some(Self::Waiting),
            PipeWireState::Error(message) => Some(Self::Failed(message)),
            PipeWireState::Unconnected => None,
        }
    }
}

/// Which node the capture is (or will be) connected to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Route {
    Raw,
    Enhanced { backend: String },
    Fallback(UnenhancedReason),
}

pub(super) fn stream_state(capture: &CaptureStatus, route: &Route) -> StreamState {
    match (capture, route) {
        (CaptureStatus::Failed(message), _) => StreamState::Failed(message.clone()),
        (CaptureStatus::Waiting, _) => StreamState::Starting,
        (CaptureStatus::Streaming, Route::Raw) => StreamState::Live { enhanced_by: None },
        (CaptureStatus::Streaming, Route::Enhanced { backend }) => StreamState::Live {
            enhanced_by: Some(backend.clone()),
        },
        (CaptureStatus::Streaming, Route::Fallback(reason)) => {
            StreamState::Unenhanced(reason.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipewire_states_map_to_capture_status() {
        assert_eq!(
            CaptureStatus::from_pipewire(PipeWireState::Streaming),
            Some(CaptureStatus::Streaming)
        );
        assert_eq!(
            CaptureStatus::from_pipewire(PipeWireState::Paused),
            Some(CaptureStatus::Waiting)
        );
        assert_eq!(
            CaptureStatus::from_pipewire(PipeWireState::Error("gone".into())),
            Some(CaptureStatus::Failed("gone".into()))
        );
        assert_eq!(
            CaptureStatus::from_pipewire(PipeWireState::Unconnected),
            None
        );
    }

    #[test]
    fn the_stream_starts_until_audio_flows_whatever_the_route() {
        for route in [
            Route::Raw,
            Route::Enhanced {
                backend: "cpu".into(),
            },
            Route::Fallback(UnenhancedReason::TimedOut),
        ] {
            assert_eq!(
                stream_state(&CaptureStatus::Waiting, &route),
                StreamState::Starting
            );
        }
    }

    #[test]
    fn streaming_reports_how_the_audio_is_enhanced() {
        let streaming = CaptureStatus::Streaming;
        assert_eq!(
            stream_state(&streaming, &Route::Raw),
            StreamState::Live { enhanced_by: None }
        );
        assert_eq!(
            stream_state(
                &streaming,
                &Route::Enhanced {
                    backend: "cpu: DeepFilterNet3".into()
                }
            ),
            StreamState::Live {
                enhanced_by: Some("cpu: DeepFilterNet3".into())
            }
        );
        assert_eq!(
            stream_state(&streaming, &Route::Fallback(UnenhancedReason::Disconnected)),
            StreamState::Unenhanced(UnenhancedReason::Disconnected)
        );
    }

    #[test]
    fn a_failed_capture_fails_the_stream() {
        assert_eq!(
            stream_state(&CaptureStatus::Failed("no device".into()), &Route::Raw),
            StreamState::Failed("no device".into())
        );
    }
}
