use std::{
    sync::mpsc::{self, SyncSender},
    thread::{self, JoinHandle},
    time::Instant,
};

use pipewire::{
    channel,
    core::Core,
    loop_::{Loop, Timeout},
};
use tokio::sync::watch;

use crate::{
    AudioError, MicrophoneOptions, StreamState,
    util::{mono_format::mono_format, pipewire_core::run_on_core},
};

use super::{
    event::{Command, Event},
    frame_sink::FrameSink,
    session::Session,
};

const THREAD_NAME: &str = "crownui-microphone";

/// The PipeWire loop thread of one microphone stream, stopped and joined on drop.
pub(crate) struct CaptureThread {
    commands: channel::Sender<Command>,
    thread: Option<JoinHandle<()>>,
}

impl CaptureThread {
    pub(crate) fn spawn(
        options: MicrophoneOptions,
        sink: FrameSink,
        state: watch::Sender<StreamState>,
    ) -> Result<Self, AudioError> {
        let (commands, received) = channel::channel();
        let (ready_sender, ready) = mpsc::sync_channel(1);
        let thread = thread::Builder::new()
            .name(THREAD_NAME.into())
            .spawn(move || {
                let outcome = run_on_core(|loop_, core| {
                    run(loop_, core, options, sink, &state, received, &ready_sender)
                });
                if let Err(error) = outcome {
                    state.send_replace(StreamState::Failed(error.to_string()));
                    let _ = ready_sender.send(Err(error));
                }
            })
            .map_err(AudioError::Thread)?;
        let spawned = Self {
            commands,
            thread: Some(thread),
        };
        match ready.recv() {
            Ok(Ok(())) => Ok(spawned),
            Ok(Err(error)) => Err(error),
            Err(_) => Err(AudioError::Stopped),
        }
    }

    pub(crate) fn send(&self, command: Command) {
        let _ = self.commands.send(command);
    }
}

impl Drop for CaptureThread {
    fn drop(&mut self) {
        self.send(Command::Stop);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn run(
    loop_: &Loop,
    core: &Core,
    options: MicrophoneOptions,
    sink: FrameSink,
    state: &watch::Sender<StreamState>,
    commands: channel::Receiver<Command>,
    ready: &SyncSender<Result<(), AudioError>>,
) -> Result<(), AudioError> {
    let format = mono_format()?;
    let (events, received) = mpsc::channel();
    let _commands = commands.attach(loop_, {
        let events = events.clone();
        move |command| {
            let _ = events.send(Event::Command(command));
        }
    });
    let mut session = Session::new(loop_, core, &format, events, state, options, sink);
    let _ = ready.send(Ok(()));
    session.start();
    loop {
        loop_.iterate(session.deadline().map_or(Timeout::Infinite, |deadline| {
            Timeout::Finite(deadline.saturating_duration_since(Instant::now()))
        }));
        for event in received.try_iter() {
            if session.handle(event).is_break() {
                return Ok(());
            }
        }
        session.check_deadline();
    }
}
