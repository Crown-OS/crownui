use std::os::fd::{AsFd, BorrowedFd, OwnedFd};

use rustix::event::{EventfdFlags, eventfd};

/// The side of a stop request that a `poll` waits on.
#[derive(Debug)]
pub(crate) struct StopSignal(OwnedFd);

/// The side of a stop request that another thread raises.
#[derive(Debug)]
pub(crate) struct StopRaiser(OwnedFd);

/// Both sides of one stop request: an eventfd that is never drained, so once raised it stays
/// raised.
pub(crate) fn stop_signal() -> std::io::Result<(StopSignal, StopRaiser)> {
    let signal = eventfd(0, EventfdFlags::CLOEXEC | EventfdFlags::NONBLOCK)?;
    let raiser = signal.try_clone()?;
    Ok((StopSignal(signal), StopRaiser(raiser)))
}

impl StopRaiser {
    pub(crate) fn raise(&self) {
        let _ = rustix::io::write(&self.0, &1u64.to_ne_bytes());
    }
}

impl AsFd for StopSignal {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}
