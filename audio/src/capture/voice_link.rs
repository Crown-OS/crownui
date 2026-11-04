use std::{
    os::fd::{AsFd, AsRawFd, RawFd},
    sync::mpsc,
    time::{Duration, Instant},
};

use crownos_ipc::{Error as IpcError, Pending};
use pipewire::{
    loop_::{IoSource, Loop},
    spa::support::system::IoFlags,
};
use voice_protocol::{EnhancedNode, StreamId, voice};

use crate::{Enhancement, SourceId, UnenhancedReason};

use super::event::Event;

const ANSWER_TIMEOUT: Duration = Duration::from_secs(3);

/// The connection to crown-voiced serving one enhanced stream. Dropping it closes the
/// connection, which makes the daemon tear the enhanced node down.
pub(super) struct VoiceLink<'l> {
    _readable: IoSource<'l, RawFd>,
    client: voice::Client,
    stream: Option<StreamId>,
    pending: Option<Pending<EnhancedNode>>,
    deadline: Option<Instant>,
    requested: Enhancement,
    desired: Enhancement,
}

impl<'l> VoiceLink<'l> {
    /// Asks crown-voiced to enhance `source`; the node arrives through [`Self::pump`].
    pub(super) fn open(
        loop_: &'l Loop,
        events: &mpsc::Sender<Event>,
        source: Option<&SourceId>,
        enhancement: Enhancement,
    ) -> Result<Self, UnenhancedReason> {
        let mut client = voice::Client::connect()
            .map_err(|error| UnenhancedReason::Unavailable(error.to_string()))?;
        drop(
            client
                .subscribe::<voice::StreamChanged>()
                .map_err(unenhanced)?,
        );
        let pending = client
            .open_stream(source.map(|source| source.as_str().to_owned()), enhancement)
            .map_err(unenhanced)?;
        let events = events.clone();
        let readable = loop_.add_io(client.as_fd().as_raw_fd(), IoFlags::IN, move |_| {
            let _ = events.send(Event::VoiceReadable);
        });
        Ok(Self {
            _readable: readable,
            client,
            stream: None,
            pending: Some(pending),
            deadline: Some(Instant::now() + ANSWER_TIMEOUT),
            requested: enhancement,
            desired: enhancement,
        })
    }

    pub(super) fn deadline(&self) -> Option<Instant> {
        self.deadline
    }

    pub(super) fn set_enhancement(
        &mut self,
        enhancement: Enhancement,
    ) -> Result<(), UnenhancedReason> {
        self.desired = enhancement;
        match self.pending {
            Some(_) => Ok(()),
            None => self.request_desired(),
        }
    }

    /// Reads what crown-voiced sent; returns the node to capture from when it changed.
    pub(super) fn pump(&mut self) -> Result<Option<EnhancedNode>, UnenhancedReason> {
        self.client.handle_readable().map_err(unenhanced)?;
        let mut latest = self.take_reply()?;
        while let Some(event) = self.client.next_event() {
            if let voice::Event::StreamChanged(changed) = event
                && Some(changed.node.stream) == self.stream
            {
                latest = Some(changed.node);
            }
        }
        Ok(latest)
    }

    fn take_reply(&mut self) -> Result<Option<EnhancedNode>, UnenhancedReason> {
        let Some(pending) = &self.pending else {
            return Ok(None);
        };
        let Some(node) = self.client.take(pending).map_err(unenhanced)? else {
            return Ok(None);
        };
        self.pending = None;
        self.deadline = None;
        self.stream = Some(node.stream);
        self.request_desired()?;
        Ok(Some(node))
    }

    fn request_desired(&mut self) -> Result<(), UnenhancedReason> {
        let Some(stream) = self.stream else {
            return Ok(());
        };
        if self.desired == self.requested {
            return Ok(());
        }
        self.pending = Some(
            self.client
                .set_enhancement(stream, self.desired)
                .map_err(unenhanced)?,
        );
        self.requested = self.desired;
        self.deadline = Some(Instant::now() + ANSWER_TIMEOUT);
        Ok(())
    }
}

fn unenhanced(error: IpcError) -> UnenhancedReason {
    match error {
        IpcError::Remote(remote) => UnenhancedReason::Refused(remote.message),
        IpcError::Disconnected => UnenhancedReason::Disconnected,
        IpcError::Timeout => UnenhancedReason::TimedOut,
        other => UnenhancedReason::Unavailable(other.to_string()),
    }
}
