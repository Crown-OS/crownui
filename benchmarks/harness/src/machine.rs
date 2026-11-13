use std::{fs, process::Command};

use serde::Serialize;

/// Conditions a result was measured under; wall-clock figures only compare within one machine.
#[derive(Debug, Clone, Serialize)]
pub struct Machine {
    pub date: String,
    pub kernel: String,
    pub cpu: String,
    pub cpu_governor: String,
    pub load_average: String,
}

impl Machine {
    pub fn describe() -> Self {
        let read = |path: &str| {
            fs::read_to_string(path)
                .map(|text| text.trim().to_owned())
                .unwrap_or_default()
        };
        let cpuinfo = read("/proc/cpuinfo");
        Self {
            date: Command::new("date")
                .args(["-u", "+%Y-%m-%dT%H-%MZ"])
                .output()
                .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
                .unwrap_or_default(),
            kernel: read("/proc/sys/kernel/osrelease"),
            cpu: cpuinfo
                .lines()
                .find_map(|line| line.strip_prefix("model name"))
                .and_then(|line| line.split_once(':'))
                .map(|(_, name)| name.trim().to_owned())
                .unwrap_or_default(),
            cpu_governor: read("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor"),
            load_average: read("/proc/loadavg"),
        }
    }
}
