use std::{fs, ops::Sub};

use crate::util::proc_fs::{field_value, stat_fields};

const USER_TIME_FIELD: usize = 11;
const SYSTEM_TIME_FIELD: usize = 12;

/// CPU time and context switches a process tree has accumulated so far; subtract two samples
/// to get the cost of the time between them.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CpuUsage {
    pub cpu_ms: f64,
    pub context_switches: u64,
}

impl CpuUsage {
    pub fn of(pids: &[u32]) -> Self {
        let millis_per_tick = 1000.0 / clock_ticks_per_second();
        pids.iter().fold(Self::default(), |total, pid| {
            let ticks: u64 = stat_fields(*pid)
                .map(|fields| {
                    [USER_TIME_FIELD, SYSTEM_TIME_FIELD]
                        .iter()
                        .filter_map(|index| fields.get(*index)?.parse::<u64>().ok())
                        .sum()
                })
                .unwrap_or(0);
            Self {
                cpu_ms: total.cpu_ms + ticks as f64 * millis_per_tick,
                context_switches: total.context_switches + context_switches(*pid),
            }
        })
    }
}

impl Sub for CpuUsage {
    type Output = Self;

    fn sub(self, earlier: Self) -> Self {
        Self {
            cpu_ms: (self.cpu_ms - earlier.cpu_ms).max(0.0),
            context_switches: self
                .context_switches
                .saturating_sub(earlier.context_switches),
        }
    }
}

/// Voluntary and involuntary switches of every thread: each one is a wakeup or a preemption.
fn context_switches(pid: u32) -> u64 {
    fs::read_dir(format!("/proc/{pid}/task"))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|thread| fs::read_to_string(thread.path().join("status")).ok())
        .map(|status| {
            field_value(&status, "voluntary_ctxt_switches").unwrap_or(0)
                + field_value(&status, "nonvoluntary_ctxt_switches").unwrap_or(0)
        })
        .sum()
}

fn clock_ticks_per_second() -> f64 {
    // SAFETY: sysconf only reads a configuration value.
    let ticks = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    if ticks > 0 { ticks as f64 } else { 100.0 }
}
