use audio::{
    Enhancement, FrameConsumer, Level, Microphone, MicrophoneOptions, MicrophoneStream, StreamState,
};
use reactive::{Signal, Store};
use view::Cx;

use super::follow::follow_watch;

/// A microphone owned by the component that called [`use_microphone`]: its level and state as
/// signals, stopped when that component unmounts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MicrophoneSignals {
    /// The level of the latest frame.
    pub level: Signal<Level>,
    /// Starting, live (and how it is enhanced), unenhanced or failed.
    pub state: Signal<StreamState>,
    stream: Store<Option<MicrophoneStream>>,
}

impl MicrophoneSignals {
    /// The frame ring, once: move it to the worker that records or analyses the audio.
    pub fn take_frames(self, cx: &mut Cx) -> Option<FrameConsumer> {
        self.stream
            .try_update(cx, |stream| {
                stream.as_mut().and_then(MicrophoneStream::take_frames)
            })
            .flatten()
    }

    /// Switches enhancement without stopping the stream; [`Self::state`] follows.
    pub fn set_enhancement(self, cx: &mut Cx, enhancement: Enhancement) {
        self.stream.try_update(cx, |stream| {
            if let Some(stream) = stream {
                stream.set_enhancement(enhancement);
            }
        });
    }
}

/// Opens a microphone for the current component and mirrors its level and state into signals.
/// A microphone that cannot open shows as [`StreamState::Failed`].
pub fn use_microphone(cx: &mut Cx, options: MicrophoneOptions) -> MicrophoneSignals {
    let (level, state, stream) = match Microphone::open(options) {
        Ok(stream) => {
            let level = cx.signal(*stream.level().borrow());
            let state = cx.signal(stream.state().borrow().clone());
            follow_watch(cx, level, stream.level());
            follow_watch(cx, state, stream.state());
            (level, state, Some(stream))
        }
        Err(error) => (
            cx.signal(Level::default()),
            cx.signal(StreamState::Failed(error.to_string())),
            None,
        ),
    };
    MicrophoneSignals {
        level,
        state,
        stream: cx.store(stream),
    }
}
