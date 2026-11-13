use std::fs;

use serde::Serialize;

use crate::util::proc_fs::field_value;

/// Resident memory summed over a process tree. PSS splits shared pages between the processes
/// that map them, so it is the fair figure for comparing toolkits that share system libraries.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct MemoryUsage {
    pub pss_kib: u64,
    pub rss_kib: u64,
    pub uss_kib: u64,
}

impl MemoryUsage {
    pub fn of(pids: &[u32]) -> Self {
        pids.iter()
            .filter_map(|pid| fs::read_to_string(format!("/proc/{pid}/smaps_rollup")).ok())
            .map(|rollup| Self {
                pss_kib: field_value(&rollup, "Pss").unwrap_or(0),
                rss_kib: field_value(&rollup, "Rss").unwrap_or(0),
                uss_kib: field_value(&rollup, "Private_Clean").unwrap_or(0)
                    + field_value(&rollup, "Private_Dirty").unwrap_or(0),
            })
            .fold(Self::default(), |total, process| Self {
                pss_kib: total.pss_kib + process.pss_kib,
                rss_kib: total.rss_kib + process.rss_kib,
                uss_kib: total.uss_kib + process.uss_kib,
            })
    }

    pub fn peak(self, other: Self) -> Self {
        Self {
            pss_kib: self.pss_kib.max(other.pss_kib),
            rss_kib: self.rss_kib.max(other.rss_kib),
            uss_kib: self.uss_kib.max(other.uss_kib),
        }
    }
}
