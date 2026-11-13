use std::{fmt::Write, fs, path::PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::{
    environment::Placement,
    footprint::Footprint,
    frame_rate::FrameRate,
    framework::Framework,
    machine::Machine,
    measure::{Measurement, MotionCost, Timing},
    scenario::Scenario,
    spec::Spec,
    util::stats::{median, percentile},
};

const KIB_PER_MIB: f64 = 1024.0;
const BYTES_PER_MIB: f64 = 1024.0 * 1024.0;

#[derive(Debug, Serialize)]
pub struct Failure {
    pub framework: Framework,
    pub scenario: Scenario,
    pub run: usize,
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct Results {
    pub machine: Machine,
    pub placement: Placement,
    pub frame_rate: FrameRate,
    pub compositor_hz: Option<f64>,
    pub spec: Spec,
    pub timing: Timing,
    pub runs: usize,
    pub frameworks: Vec<Framework>,
    pub footprints: Vec<(Framework, Footprint)>,
    pub measurements: Vec<Measurement>,
    pub failures: Vec<Failure>,
}

impl Results {
    /// Writes `<date>.json` and `<date>.md` into `directory` and returns the markdown path.
    pub fn write(&self, directory: PathBuf) -> Result<PathBuf> {
        fs::create_dir_all(&directory)?;
        let stem = directory.join(&self.machine.date);
        let json = stem.with_extension("json");
        fs::write(&json, serde_json::to_string_pretty(self)?)
            .with_context(|| format!("writing {}", json.display()))?;
        let markdown = stem.with_extension("md");
        fs::write(&markdown, self.markdown())
            .with_context(|| format!("writing {}", markdown.display()))?;
        Ok(markdown)
    }

    pub fn markdown(&self) -> String {
        let mut out = String::new();
        let machine = &self.machine;
        let _ = writeln!(out, "# CrownUI vs other toolkits: {}\n", machine.date);
        let _ = writeln!(
            out,
            "{} · kernel {} · governor `{}` · load `{}` · `{}` placement · {} · output mode {} · compositor pace {} · {} runs × {} s (+{} s settle), medians\n",
            machine.cpu,
            machine.kernel,
            machine.cpu_governor,
            machine.load_average,
            self.placement.name(),
            match self.frame_rate {
                FrameRate::Display => "frames paced by the display",
                FrameRate::Uncapped => "frame pacing off (`--uncapped`)",
            },
            hertz(self.measurements.iter().find_map(|run| run.refresh_hz)),
            hertz(self.compositor_hz),
            self.runs,
            self.timing.duration.as_secs_f64(),
            self.timing.settle.as_secs_f64(),
        );
        self.write_idle_table(&mut out);
        for scenario in [Scenario::List, Scenario::Animation, Scenario::Settings] {
            self.write_motion_table(&mut out, scenario);
        }
        if !self.failures.is_empty() {
            let _ = writeln!(out, "## Failures\n");
            for failure in &self.failures {
                let _ = writeln!(
                    out,
                    "- {} {} run {}: {}",
                    failure.framework.name(),
                    failure.scenario.name(),
                    failure.run,
                    failure.error.lines().next().unwrap_or_default()
                );
            }
        }
        out
    }

    fn write_idle_table(&self, out: &mut String) {
        if !self
            .measurements
            .iter()
            .any(|run| run.scenario == Scenario::Hello)
        {
            return;
        }
        let _ = writeln!(out, "## Startup, memory and idle cost (hello)\n");
        let _ = writeln!(
            out,
            "| Framework | Startup ms (p50) | Startup ms (p90) | PSS MiB | USS MiB | RSS MiB | VRAM MiB | GTT MiB | Idle CPU ms/s | Idle wakeups/s | Idle frames | Own files MiB | Libraries MiB |"
        );
        let _ = writeln!(out, "|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|");
        for framework in &self.frameworks {
            let runs = self.runs_of(*framework, Scenario::Hello);
            if runs.is_empty() {
                continue;
            }
            let startup: Vec<f64> = runs.iter().map(|run| run.startup_ms).collect();
            let footprint = self
                .footprints
                .iter()
                .find(|(owner, _)| owner == framework)
                .map(|(_, footprint)| *footprint);
            let idle = |field: fn(&Measurement) -> Option<f64>| cell(median_of(&runs, field), 1);
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                framework.name(),
                cell(median(&startup), 0),
                cell(percentile(&startup, 0.9), 0),
                idle(|run| Some(run.memory.pss_kib as f64 / KIB_PER_MIB)),
                idle(|run| Some(run.memory.uss_kib as f64 / KIB_PER_MIB)),
                idle(|run| Some(run.memory.rss_kib as f64 / KIB_PER_MIB)),
                idle(|run| Some(run.gpu.vram_kib as f64 / KIB_PER_MIB)),
                idle(|run| Some(run.gpu.gtt_kib as f64 / KIB_PER_MIB)),
                idle(|run| run.idle.map(|idle| idle.cpu_ms_per_second)),
                idle(|run| run.idle.map(|idle| idle.wakeups_per_second)),
                idle(|run| run.idle.map(|idle| idle.frames as f64)),
                cell(footprint.map(|f| f.own_bytes as f64 / BYTES_PER_MIB), 1),
                cell(footprint.map(|f| f.library_bytes as f64 / BYTES_PER_MIB), 1),
            );
        }
        let _ = writeln!(out);
    }

    fn write_motion_table(&self, out: &mut String, scenario: Scenario) {
        if !self.measurements.iter().any(|run| run.scenario == scenario) {
            return;
        }
        let _ = writeln!(out, "## {}\n", heading(scenario));
        let _ = writeln!(
            out,
            "| Framework | FPS | Frame p50 ms | Frame p99 ms | Long frames % | CPU % | CPU ms/frame | M instr/frame | M cycles/frame | Peak PSS MiB | Peak VRAM MiB |"
        );
        let _ = writeln!(out, "|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|");
        for framework in &self.frameworks {
            let runs = self.runs_of(*framework, scenario);
            if runs.is_empty() {
                continue;
            }
            let motion = |field: fn(&MotionCost) -> Option<f64>, digits| {
                cell(
                    median_of(&runs, |run| run.motion.as_ref().and_then(field)),
                    digits,
                )
            };
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                framework.name(),
                motion(|cost| Some(cost.fps), 1),
                motion(|cost| cost.interval_p50_ms, 2),
                motion(|cost| cost.interval_p99_ms, 2),
                motion(|cost| cost.long_frames_percent, 1),
                motion(|cost| cost.cpu_percent, 1),
                motion(|cost| cost.cpu_ms_per_frame, 2),
                motion(
                    |cost| cost.instructions_per_frame.map(|count| count / 1e6),
                    2
                ),
                motion(|cost| cost.cycles_per_frame.map(|count| count / 1e6), 2),
                cell(
                    median_of(&runs, |run| Some(run.memory.pss_kib as f64 / KIB_PER_MIB)),
                    1
                ),
                cell(
                    median_of(&runs, |run| Some(run.gpu.vram_kib as f64 / KIB_PER_MIB)),
                    1
                ),
            );
        }
        let note = match self.frame_rate {
            FrameRate::Display => {
                "Frames above the compositor pace are drawn but never shown. Long frames take over 1.5 compositor refreshes. Cycles are user-space and do not depend on the clock the governor picks, which CPU ms do."
            }
            FrameRate::Uncapped => {
                "Frame pacing is off for CrownUI and GPUI, so their FPS is how many frames they can draw; the compositor shows at most one per refresh. Qt, Electron and Tauri have no switch that turns it off on Wayland and run paced as usual."
            }
        };
        let _ = writeln!(out, "\n{note}\n");
    }

    fn runs_of(&self, framework: Framework, scenario: Scenario) -> Vec<&Measurement> {
        self.measurements
            .iter()
            .filter(|run| run.framework == framework && run.scenario == scenario)
            .collect()
    }
}

fn hertz(value: Option<f64>) -> String {
    value.map_or_else(|| "unknown".to_owned(), |hz| format!("{hz:.0} Hz"))
}

fn heading(scenario: Scenario) -> &'static str {
    match scenario {
        Scenario::Hello => "Hello",
        Scenario::List => "Scrolling a 10k-row list",
        Scenario::Animation => "Animation and per-frame text updates",
        Scenario::Settings => "A settings window: scrolling pages of cards and switching pages",
    }
}

fn median_of(runs: &[&Measurement], field: impl Fn(&Measurement) -> Option<f64>) -> Option<f64> {
    let values: Vec<f64> = runs.iter().filter_map(|run| field(run)).collect();
    median(&values)
}

fn cell(value: Option<f64>, digits: usize) -> String {
    value.map_or_else(|| "—".to_owned(), |value| format!("{value:.digits$}"))
}
