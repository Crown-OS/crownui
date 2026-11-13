use std::{collections::HashSet, fs};

use serde::Serialize;

use crate::util::proc_fs::field_value;

/// GPU memory a process tree holds, from the DRM fdinfo every open render node exposes. Each
/// DRM client is counted once, however many descriptors or processes share it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct GpuMemory {
    pub vram_kib: u64,
    pub gtt_kib: u64,
}

impl GpuMemory {
    pub fn of(pids: &[u32]) -> Self {
        let mut clients = HashSet::new();
        let mut total = Self::default();
        for pid in pids {
            let Ok(descriptors) = fs::read_dir(format!("/proc/{pid}/fdinfo")) else {
                continue;
            };
            for info in descriptors
                .flatten()
                .filter_map(|entry| fs::read_to_string(entry.path()).ok())
            {
                let Some(client) = field_value(&info, "drm-client-id") else {
                    continue;
                };
                let device = info
                    .lines()
                    .find_map(|line| line.strip_prefix("drm-pdev:"))
                    .map(|device| device.trim().to_owned());
                if clients.insert((device, client)) {
                    total.vram_kib += field_value(&info, "drm-total-vram")
                        .or_else(|| field_value(&info, "drm-memory-vram"))
                        .unwrap_or(0);
                    total.gtt_kib += field_value(&info, "drm-total-gtt")
                        .or_else(|| field_value(&info, "drm-memory-gtt"))
                        .unwrap_or(0);
                }
            }
        }
        total
    }

    pub fn peak(self, other: Self) -> Self {
        Self {
            vram_kib: self.vram_kib.max(other.vram_kib),
            gtt_kib: self.gtt_kib.max(other.gtt_kib),
        }
    }
}
