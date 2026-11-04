use crate::Enhancement;

use super::status::CaptureStatus;

/// What a [`MicrophoneStream`](crate::MicrophoneStream) asks of its capture thread.
pub(crate) enum Command {
    SetEnhancement(Enhancement),
    Stop,
}

/// Everything the capture thread reacts to, funnelled through one queue it owns.
pub(super) enum Event {
    Command(Command),
    Capture {
        generation: u32,
        status: CaptureStatus,
    },
    VoiceReadable,
}
