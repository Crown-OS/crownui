use std::{
    ffi::CString,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use serde::Serialize;

const EVENTS: &str = "instructions:u,cycles:u,task-clock";

/// Counters `perf stat` collected while enabled, summed over every process and thread of the
/// app (children inherit the counters).
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
pub struct PerfCounters {
    pub instructions: Option<u64>,
    pub cycles: Option<u64>,
    pub task_clock_ms: Option<f64>,
}

/// A `perf stat` run that starts with its counters off and is switched on and off through a
/// control FIFO, so the counted window is exactly the measured one: no startup, no shutdown.
pub struct PerfSession {
    control_path: PathBuf,
    output: PathBuf,
    control: Option<File>,
}

impl PerfSession {
    pub fn create(directory: &Path, name: &str) -> Result<Self> {
        fs::create_dir_all(directory)?;
        let control_path = directory.join(format!("{name}.ctl"));
        let output = directory.join(format!("{name}.csv"));
        for stale in [&control_path, &output] {
            let _ = fs::remove_file(stale);
        }
        make_fifo(&control_path).with_context(|| format!("creating {}", control_path.display()))?;
        Ok(Self {
            control_path,
            output,
            control: None,
        })
    }

    pub fn wrap(&self, inner: &Command) -> Command {
        let mut perf = Command::new("perf");
        perf.args(["stat", "-x", ",", "-e", EVENTS, "--delay=-1"])
            .arg(format!("--control=fifo:{}", self.control_path.display()))
            .arg("-o")
            .arg(&self.output)
            .arg("--")
            .arg(inner.get_program())
            .args(inner.get_args());
        perf
    }

    pub fn enable(&mut self) -> Result<()> {
        self.send("enable")
    }

    pub fn disable(&mut self) -> Result<()> {
        self.send("disable")
    }

    /// Reads the counters; call once perf has exited.
    pub fn counters(&self) -> PerfCounters {
        fs::read_to_string(&self.output)
            .map(|csv| PerfCounters::parse(&csv))
            .unwrap_or_default()
    }

    fn send(&mut self, command: &str) -> Result<()> {
        if self.control.is_none() {
            // Read-write, so opening never blocks waiting for perf to open its end.
            self.control = Some(
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(&self.control_path)?,
            );
        }
        let control = self.control.as_mut().context("the control FIFO is open")?;
        writeln!(control, "{command}").context("signalling perf")
    }
}

impl Drop for PerfSession {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.control_path);
    }
}

fn make_fifo(path: &Path) -> io::Result<()> {
    let path = CString::new(path.as_os_str().as_bytes())?;
    // SAFETY: `path` is a valid NUL-terminated string for the duration of the call.
    if unsafe { libc::mkfifo(path.as_ptr(), 0o600) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

impl PerfCounters {
    pub fn parse(csv: &str) -> Self {
        let mut counters = Self::default();
        for line in csv.lines().filter(|line| !line.starts_with('#')) {
            let fields: Vec<&str> = line.split(',').collect();
            let (Some(value), Some(event)) = (fields.first(), fields.get(2)) else {
                continue;
            };
            if event.starts_with("instructions") {
                counters.instructions = value.parse().ok();
            } else if event.starts_with("cycles") {
                counters.cycles = value.parse().ok();
            } else if event.starts_with("task-clock") {
                counters.task_clock_ms = value.parse().ok();
            }
        }
        counters
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_perf_csv() {
        let csv = "# started on Sat Nov  7 02:00:00 2026\n\n\
                   1234.56,msec,task-clock,1234560000,100.00,0.957,CPUs utilized\n\
                   987654321,,instructions:u,1234560000,100.00,1.23,insn per cycle\n\
                   <not counted>,,cycles:u,0,0.00,,\n";
        assert_eq!(
            PerfCounters::parse(csv),
            PerfCounters {
                instructions: Some(987_654_321),
                cycles: None,
                task_clock_ms: Some(1234.56),
            }
        );
    }
}
