use sctk::reexports::{
    calloop,
    client::{ConnectError, DispatchError, globals::BindError, globals::GlobalError},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to connect to the Wayland compositor: {0}")]
    Connect(#[from] ConnectError),
    #[error("failed to read the compositor's globals: {0}")]
    Globals(#[from] GlobalError),
    #[error("the compositor does not provide {interface}: {source}")]
    RequiredGlobal {
        interface: &'static str,
        source: BindError,
    },
    #[error("the compositor does not provide {0}")]
    MissingGlobal(&'static str),
    #[error("Wayland dispatch failed: {0}")]
    Dispatch(#[from] DispatchError),
    #[error("event loop failure: {0}")]
    EventLoop(#[from] calloop::Error),
    #[error("failed to create the event loop waker: {0}")]
    Waker(#[from] std::io::Error),
    #[error("the compositor has no seat")]
    NoSeat,
    #[error("virtual output name {0:?} must be 1 to 32 characters from [A-Za-z0-9_-]")]
    InvalidVirtualOutputName(String),
    #[error("virtual output mode is out of range")]
    InvalidVirtualOutputMode,
    #[error("screencast slot {0} is outside the ring")]
    InvalidScreencastSlot(u32),
    #[error("the screencast session was already started")]
    ScreencastAlreadyStarted,
    #[error("timelines can only be set once, before the session starts")]
    ScreencastTimelinesRejected,
    #[error("the compositor does not support explicit sync for screencasts")]
    ExplicitSyncUnsupported,
}
