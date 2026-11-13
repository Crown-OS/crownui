use std::{
    collections::VecDeque,
    io::{BufRead, BufReader},
    os::unix::process::CommandExt,
    process::{Child, ChildStderr, Command, Stdio},
    sync::mpsc::{self, Receiver, RecvTimeoutError},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};

use crate::{
    process_tree,
    wayland_trace::{FrameTracker, TraceEvent},
};

const STOP_GRACE: Duration = Duration::from_secs(5);
const STOP_POLL: Duration = Duration::from_millis(20);
const KEPT_LOG_LINES: usize = 12;

/// The compositor does not show the app's window, so nothing it measures means anything; every
/// later run would fail the same way.
#[derive(Debug)]
pub struct WindowNotShown;

impl std::fmt::Display for WindowNotShown {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(
            "the compositor does not show the window: unlock the screen and keep it awake",
        )
    }
}

impl std::error::Error for WindowNotShown {}

/// A running app whose protocol trace is read on a separate thread and stamped on arrival.
pub struct Session {
    child: Child,
    spawned_at: Instant,
    observations: Receiver<(Instant, TraceEvent)>,
    reader: Option<JoinHandle<VecDeque<String>>>,
    frames: Vec<Instant>,
    frame_callbacks: Vec<(Instant, u32)>,
    refresh_millihertz: Option<u32>,
    not_shown: bool,
    wrapped: bool,
}

impl Session {
    /// Starts `command` in its own process group. `wrapped` says the first process is a
    /// launcher (`perf`) that should not count as part of the app.
    pub fn launch(mut command: Command, wrapped: bool) -> Result<Self> {
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .process_group(0);
        let spawned_at = Instant::now();
        let mut child = command
            .spawn()
            .with_context(|| format!("starting {command:?}"))?;
        let stderr = child.stderr.take().context("stderr is piped")?;
        let (sender, observations) = mpsc::channel();
        let reader = thread::spawn(move || {
            read_trace(stderr, |event| {
                let _ = sender.send((Instant::now(), event));
            })
        });
        Ok(Self {
            child,
            spawned_at,
            observations,
            reader: Some(reader),
            frames: Vec::new(),
            frame_callbacks: Vec::new(),
            refresh_millihertz: None,
            not_shown: false,
            wrapped,
        })
    }

    pub fn refresh_millihertz(&self) -> Option<u32> {
        self.refresh_millihertz
    }

    /// Time from spawn to the first frame.
    pub fn first_frame(&mut self, timeout: Duration) -> Result<Duration> {
        let deadline = self.spawned_at + timeout;
        while self.frames.is_empty() {
            if self.not_shown {
                self.stop_and_collect_log();
                return Err(WindowNotShown.into());
            }
            if !self.receive_until(deadline) {
                let log = self.stop_and_collect_log();
                bail!(
                    "no frame within {timeout:?}: the app crashed or hung, is not on native Wayland, or its window is hidden (screen locked?). Last output:\n{log}"
                );
            }
        }
        Ok(self.frames[0] - self.spawned_at)
    }

    /// Records trace events until `deadline`.
    pub fn wait_until(&mut self, deadline: Instant) {
        while self.receive_until(deadline) {}
        if let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
            thread::sleep(remaining);
        }
    }

    pub fn frames_between(&self, start: Instant, end: Instant) -> Vec<Instant> {
        self.frames
            .iter()
            .copied()
            .filter(|frame| (start..end).contains(frame))
            .collect()
    }

    /// The compositor's clock (milliseconds) at each frame callback it sent in the window.
    pub fn frame_callbacks_between(&self, start: Instant, end: Instant) -> Vec<u32> {
        self.frame_callbacks
            .iter()
            .filter(|(at, _)| (start..end).contains(at))
            .map(|(_, clock)| *clock)
            .collect()
    }

    pub fn app_pids(&self) -> Vec<u32> {
        let mut pids = process_tree::descendants(self.child.id());
        if self.wrapped {
            pids.remove(0);
        }
        pids
    }

    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Interrupts the whole process group (which also makes `perf` write its counters), then
    /// kills whatever is left after a grace period.
    pub fn stop(mut self) -> Result<()> {
        self.stop_and_collect_log();
        Ok(())
    }

    fn stop_and_collect_log(&mut self) -> String {
        let group = self.child.id() as libc::pid_t;
        // SAFETY: signalling a process group we created has no memory-safety preconditions.
        unsafe { libc::killpg(group, libc::SIGINT) };
        let deadline = Instant::now() + STOP_GRACE;
        while Instant::now() < deadline && self.is_running() {
            thread::sleep(STOP_POLL);
        }
        // SAFETY: as above; the group may already be gone.
        unsafe { libc::killpg(group, libc::SIGKILL) };
        let _ = self.child.wait();
        self.reader
            .take()
            .and_then(|reader| reader.join().ok())
            .map(Vec::from)
            .unwrap_or_default()
            .join("\n")
    }

    /// Takes one observation; false once `deadline` passes or the trace ends.
    fn receive_until(&mut self, deadline: Instant) -> bool {
        let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
            return false;
        };
        match self.observations.recv_timeout(remaining) {
            Ok((at, TraceEvent::Frame)) => {
                self.frames.push(at);
                true
            }
            Ok((at, TraceEvent::FrameCallback(clock))) => {
                self.frame_callbacks.push((at, clock));
                true
            }
            Ok((_, TraceEvent::NotShown)) => {
                self.not_shown = true;
                true
            }
            Ok((_, TraceEvent::Refresh(millihertz))) => {
                self.refresh_millihertz.get_or_insert(millihertz);
                true
            }
            Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => false,
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if self.reader.is_some() {
            self.stop_and_collect_log();
        }
    }
}

/// Feeds every stderr line to the frame tracker and keeps the last few lines that are not
/// protocol trace, to explain a failed launch.
fn read_trace(stderr: ChildStderr, mut emit: impl FnMut(TraceEvent)) -> VecDeque<String> {
    let mut tracker = FrameTracker::default();
    let mut log = VecDeque::with_capacity(KEPT_LOG_LINES);
    for line in BufReader::new(stderr).lines().map_while(Result::ok) {
        match tracker.observe(&line) {
            Some(event) => emit(event),
            None if !line.starts_with('[') || !line.contains('#') => {
                if log.len() == KEPT_LOG_LINES {
                    log.pop_front();
                }
                log.push_back(line);
            }
            None => {}
        }
    }
    log
}
