use std::io::ErrorKind;

use rustix::{
    event::{PollFd, PollFlags, poll},
    io::Errno,
};
use sctk::reexports::client::{
    DispatchError, EventQueue,
    backend::{ReadEventsGuard, WaylandError},
};

use super::stop_signal::StopSignal;
use crate::wayland::error::Error;

/// Dispatches `queue` until `done` holds for `state`, blocking on the compositor's socket in
/// between. Returns [`Error::Stopped`] as soon as `stop` is raised, also while it blocks.
pub(crate) fn dispatch_until<State>(
    queue: &mut EventQueue<State>,
    state: &mut State,
    stop: &StopSignal,
    done: impl Fn(&State) -> bool,
) -> Result<(), Error> {
    loop {
        queue.dispatch_pending(state)?;
        if done(state) {
            return Ok(());
        }
        queue.flush().map_err(DispatchError::from)?;
        let Some(read) = queue.prepare_read() else {
            continue;
        };
        if stop_raised_before_events(&read, stop)? {
            return Err(Error::Stopped);
        }
        read_events(read)?;
    }
}

fn stop_raised_before_events(read: &ReadEventsGuard, stop: &StopSignal) -> Result<bool, Error> {
    let socket = read.connection_fd();
    let mut fds = [
        PollFd::new(stop, PollFlags::IN),
        PollFd::new(&socket, PollFlags::IN | PollFlags::ERR | PollFlags::HUP),
    ];
    loop {
        match poll(&mut fds, None) {
            Ok(_) => return Ok(!fds[0].revents().is_empty()),
            Err(Errno::INTR) => continue,
            Err(errno) => return Err(Error::Poll(errno.into())),
        }
    }
}

fn read_events(read: ReadEventsGuard) -> Result<(), Error> {
    match read.read() {
        Ok(_) => Ok(()),
        Err(WaylandError::Io(error)) if error.kind() == ErrorKind::WouldBlock => Ok(()),
        Err(error) => Err(DispatchError::from(error).into()),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::Write,
        os::unix::net::UnixStream,
        sync::mpsc,
        thread,
        time::{Duration, Instant},
    };

    use sctk::reexports::client::{
        Connection, Dispatch, QueueHandle,
        protocol::wl_callback::{self, WlCallback},
    };

    use super::*;
    use crate::wayland::util::stop_signal::stop_signal;

    const PROMPTLY: Duration = Duration::from_secs(2);

    #[derive(Default)]
    struct SyncDone(bool);

    impl Dispatch<WlCallback, ()> for SyncDone {
        fn event(
            state: &mut Self,
            _: &WlCallback,
            event: wl_callback::Event,
            _: &(),
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
            if let wl_callback::Event::Done { .. } = event {
                state.0 = true;
            }
        }
    }

    /// A connection to a compositor that never answers, with one `wl_display.sync` waiting.
    fn silent_compositor() -> anyhow::Result<(UnixStream, Connection, EventQueue<SyncDone>)> {
        let (client, compositor) = UnixStream::pair()?;
        let connection = Connection::from_socket(client)?;
        let queue = connection.new_event_queue();
        connection.display().sync(&queue.handle(), ());
        Ok((compositor, connection, queue))
    }

    fn wait_on_thread(
        mut queue: EventQueue<SyncDone>,
        stop: StopSignal,
    ) -> mpsc::Receiver<Result<(), Error>> {
        let (result_sender, result) = mpsc::channel();
        thread::spawn(move || {
            let mut state = SyncDone::default();
            let outcome = dispatch_until(&mut queue, &mut state, &stop, |state| state.0);
            let _ = result_sender.send(outcome);
        });
        result
    }

    #[test]
    fn a_raised_stop_ends_a_wait_the_compositor_never_answers() -> anyhow::Result<()> {
        let (_compositor, _connection, queue) = silent_compositor()?;
        let (stop, raiser) = stop_signal()?;
        let result = wait_on_thread(queue, stop);
        assert!(result.recv_timeout(Duration::from_millis(200)).is_err());

        let raised = Instant::now();
        raiser.raise();
        let outcome = result.recv_timeout(PROMPTLY)?;
        assert!(matches!(outcome, Err(Error::Stopped)), "{outcome:?}");
        assert!(raised.elapsed() < PROMPTLY);
        Ok(())
    }

    #[test]
    fn a_stop_raised_before_the_wait_still_ends_it() -> anyhow::Result<()> {
        let (_compositor, _connection, queue) = silent_compositor()?;
        let (stop, raiser) = stop_signal()?;
        raiser.raise();
        let outcome = wait_on_thread(queue, stop).recv_timeout(PROMPTLY)?;
        assert!(matches!(outcome, Err(Error::Stopped)), "{outcome:?}");
        Ok(())
    }

    #[test]
    fn the_wait_returns_when_the_compositor_answers() -> anyhow::Result<()> {
        let (mut compositor, _connection, queue) = silent_compositor()?;
        let (stop, _raiser) = stop_signal()?;
        let result = wait_on_thread(queue, stop);

        let sync_callback_id = 2u32;
        let done_opcode_and_size = 12u32 << 16;
        let callback_data = 7u32;
        for word in [sync_callback_id, done_opcode_and_size, callback_data] {
            compositor.write_all(&word.to_ne_bytes())?;
        }
        let outcome = result.recv_timeout(PROMPTLY)?;
        assert!(outcome.is_ok(), "{outcome:?}");
        Ok(())
    }
}
