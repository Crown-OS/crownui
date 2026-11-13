use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use anyhow::{Result, ensure};
use serde::Serialize;

use crate::{
    environment::{self, Placement},
    frame_rate::FrameRate,
    framework::Framework,
    paths::Paths,
    perf_counters::{PerfCounters, PerfSession},
    sample::{CpuUsage, GpuMemory, MemoryUsage},
    scenario::Scenario,
    session::{Session, WindowNotShown},
    spec::Spec,
    util::stats::percentile,
};

const FIRST_FRAME_TIMEOUT: Duration = Duration::from_secs(30);
const SAMPLE_INTERVAL: Duration = Duration::from_millis(500);
const LONG_FRAME_FACTOR: f64 = 1.5;
/// Slower than this, the compositor is throttling a window it does not show.
const THROTTLED_INTERVAL_MS: f64 = 100.0;
/// The shortest regular gap between frame callbacks is one refresh of the compositor.
const PACE_PERCENTILE: f64 = 0.1;

/// How long an app settles after its first frame, and how long it is then measured.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Timing {
    pub settle: Duration,
    pub duration: Duration,
}

#[derive(Debug, Clone, Serialize)]
pub struct Measurement {
    pub framework: Framework,
    pub scenario: Scenario,
    pub run: usize,
    pub startup_ms: f64,
    pub refresh_hz: Option<f64>,
    pub compositor_hz: Option<f64>,
    pub memory: MemoryUsage,
    pub gpu: GpuMemory,
    pub idle: Option<IdleCost>,
    pub motion: Option<MotionCost>,
}

/// What a static window costs while nothing happens.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct IdleCost {
    pub cpu_ms_per_second: f64,
    pub wakeups_per_second: f64,
    pub frames: usize,
}

/// What a continuously changing window costs.
#[derive(Debug, Clone, Serialize)]
pub struct MotionCost {
    pub frames: usize,
    pub fps: f64,
    pub interval_p50_ms: Option<f64>,
    pub interval_p99_ms: Option<f64>,
    pub long_frames_percent: Option<f64>,
    #[serde(skip)]
    pub intervals_ms: Vec<f64>,
    pub counters: PerfCounters,
    pub cpu_percent: Option<f64>,
    pub cpu_ms_per_frame: Option<f64>,
    pub instructions_per_frame: Option<f64>,
    pub cycles_per_frame: Option<f64>,
}

impl MotionCost {
    /// Frames that took longer than 1.5 refreshes of a compositor running at `pace_hz`.
    pub fn count_long_frames(&mut self, pace_hz: f64) {
        let budget = LONG_FRAME_FACTOR * 1000.0 / pace_hz;
        let long = self
            .intervals_ms
            .iter()
            .filter(|interval| **interval > budget)
            .count();
        self.long_frames_percent = (!self.intervals_ms.is_empty())
            .then(|| 100.0 * long as f64 / self.intervals_ms.len() as f64);
    }
}

pub struct Bench<'a> {
    pub paths: &'a Paths,
    pub spec: &'a Spec,
    pub timing: Timing,
    pub placement: Placement,
    pub frame_rate: FrameRate,
}

impl Bench<'_> {
    pub fn measure(
        &self,
        framework: Framework,
        scenario: Scenario,
        run: usize,
    ) -> Result<Measurement> {
        match scenario {
            Scenario::Hello => self.measure_idle(framework, run),
            Scenario::List | Scenario::Animation | Scenario::Settings => {
                self.measure_motion(framework, scenario, run)
            }
        }
    }

    fn measure_idle(&self, framework: Framework, run: usize) -> Result<Measurement> {
        let mut command = framework.command(self.paths, self.frame_rate)?;
        environment::apply(
            &mut command,
            self.spec,
            Scenario::Hello,
            self.placement,
            self.frame_rate,
        );
        let mut session = Session::launch(command, false)?;
        let startup = session.first_frame(FIRST_FRAME_TIMEOUT)?;
        let start = Instant::now() + self.timing.settle;
        session.wait_until(start);
        let pids = session.app_pids();
        let memory = MemoryUsage::of(&pids);
        let gpu = GpuMemory::of(&pids);
        let before = CpuUsage::of(&pids);
        let end = start + self.timing.duration;
        session.wait_until(end);
        let cost = CpuUsage::of(&session.app_pids()) - before;
        ensure!(
            session.is_running(),
            "the app exited during the measurement"
        );
        let seconds = self.timing.duration.as_secs_f64();
        let measurement = Measurement {
            framework,
            scenario: Scenario::Hello,
            run,
            startup_ms: millis(startup),
            refresh_hz: refresh_hz(&session),
            compositor_hz: None,
            memory,
            gpu,
            idle: Some(IdleCost {
                cpu_ms_per_second: cost.cpu_ms / seconds,
                wakeups_per_second: cost.context_switches as f64 / seconds,
                frames: session.frames_between(start, end).len(),
            }),
            motion: None,
        };
        session.stop()?;
        Ok(measurement)
    }

    fn measure_motion(
        &self,
        framework: Framework,
        scenario: Scenario,
        run: usize,
    ) -> Result<Measurement> {
        let mut perf = PerfSession::create(
            &self.scratch(),
            &format!("{}-{}-{run}", framework.name(), scenario.name()),
        )?;
        let mut command = perf.wrap(&framework.command(self.paths, self.frame_rate)?);
        environment::apply(
            &mut command,
            self.spec,
            scenario,
            self.placement,
            self.frame_rate,
        );
        let mut session = Session::launch(command, true)?;
        let startup = session.first_frame(FIRST_FRAME_TIMEOUT)?;
        let start = Instant::now() + self.timing.settle;
        session.wait_until(start);
        perf.enable()?;
        let end = start + self.timing.duration;
        let mut memory = MemoryUsage::default();
        let mut gpu = GpuMemory::default();
        while Instant::now() < end {
            session.wait_until((Instant::now() + SAMPLE_INTERVAL).min(end));
            let pids = session.app_pids();
            memory = memory.peak(MemoryUsage::of(&pids));
            gpu = gpu.peak(GpuMemory::of(&pids));
        }
        perf.disable()?;
        ensure!(
            session.is_running(),
            "the app exited during the measurement"
        );
        let frames = session.frames_between(start, end);
        let refresh_hz = refresh_hz(&session);
        let compositor_hz = compositor_pace(&session.frame_callbacks_between(start, end));
        session.stop()?;
        if compositor_hz.is_some_and(|hz| 1000.0 / hz >= THROTTLED_INTERVAL_MS) {
            return Err(WindowNotShown.into());
        }
        let counters = perf.counters();
        Ok(Measurement {
            framework,
            scenario,
            run,
            startup_ms: millis(startup),
            refresh_hz,
            compositor_hz,
            memory,
            gpu,
            idle: None,
            motion: Some(motion_cost(&frames, self.timing.duration, counters)),
        })
    }

    fn scratch(&self) -> PathBuf {
        self.paths.target.join("perf")
    }
}

/// Long frames are counted once the whole session's compositor pace is known, since apps that
/// pace with `wp_fifo` instead of frame callbacks do not reveal it.
fn motion_cost(frames: &[Instant], duration: Duration, counters: PerfCounters) -> MotionCost {
    let intervals: Vec<f64> = frames
        .windows(2)
        .map(|pair| millis(pair[1] - pair[0]))
        .collect();
    let per_frame = |total: f64| (!frames.is_empty()).then(|| total / frames.len() as f64);
    MotionCost {
        frames: frames.len(),
        fps: frames.len() as f64 / duration.as_secs_f64(),
        interval_p50_ms: percentile(&intervals, 0.5),
        interval_p99_ms: percentile(&intervals, 0.99),
        long_frames_percent: None,
        intervals_ms: intervals,
        counters,
        cpu_percent: counters
            .task_clock_ms
            .map(|cpu| 100.0 * cpu / millis(duration)),
        cpu_ms_per_frame: counters.task_clock_ms.and_then(per_frame),
        instructions_per_frame: counters
            .instructions
            .and_then(|instructions| per_frame(instructions as f64)),
        cycles_per_frame: counters.cycles.and_then(|cycles| per_frame(cycles as f64)),
    }
}

/// The rate the compositor offered frames at, from the gaps between its frame callbacks.
fn compositor_pace(callback_clocks: &[u32]) -> Option<f64> {
    let gaps: Vec<f64> = callback_clocks
        .windows(2)
        .map(|pair| f64::from(pair[1].wrapping_sub(pair[0])))
        .filter(|gap| *gap > 0.0)
        .collect();
    percentile(&gaps, PACE_PERCENTILE).map(|gap| 1000.0 / gap)
}

fn refresh_hz(session: &Session) -> Option<f64> {
    session
        .refresh_millihertz()
        .map(|millihertz| f64::from(millihertz) / 1000.0)
}

fn millis(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}
