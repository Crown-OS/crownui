#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod capture;
mod clock;
mod error;
mod frame;
mod level;
mod microphone;
mod options;
mod source;
mod sources;
mod state;
mod stream;
mod util;

pub use clock::ClockNanos;
pub use error::AudioError;
pub use frame::{AudioFrame, FRAME_SAMPLES, FrameConsumer};
pub use level::Level;
pub use microphone::Microphone;
pub use options::MicrophoneOptions;
pub use source::{SourceId, SourceInfo};
pub use sources::sources;
pub use state::{StreamState, UnenhancedReason};
pub use stream::MicrophoneStream;
pub use voice_protocol::{Enhancement, SAMPLE_RATE};
