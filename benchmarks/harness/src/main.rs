//! `crownbench [--frameworks a,b] [--scenarios a,b] [--runs N] [--duration SECONDS]
//! [--settle SECONDS] [--placement headless|desktop] [--uncapped] [--no-build]`: builds the same
//! scenarios in every framework, measures each from outside the app and writes
//! `results/<date>.{json,md}`.

mod environment;
mod footprint;
mod frame_rate;
mod framework;
mod keep_awake;
mod machine;
mod measure;
mod paths;
mod perf_counters;
mod process_tree;
mod report;
mod sample;
mod scenario;
mod session;
mod spec;
mod util;
mod wayland_trace;

use std::{str::FromStr, time::Duration};

use anyhow::{Context, Result, bail};

use crate::{
    environment::Placement,
    footprint::Footprint,
    frame_rate::FrameRate,
    framework::Framework,
    keep_awake::KeepAwake,
    machine::Machine,
    measure::{Bench, Measurement, Timing},
    paths::Paths,
    report::{Failure, Results},
    scenario::Scenario,
    session::WindowNotShown,
    spec::Spec,
    util::stats::median,
};

struct Options {
    frameworks: Vec<Framework>,
    scenarios: Vec<Scenario>,
    runs: usize,
    timing: Timing,
    placement: Placement,
    frame_rate: FrameRate,
    build: bool,
}

impl Options {
    fn parse(arguments: impl Iterator<Item = String>) -> Result<Self> {
        let mut options = Self {
            frameworks: Framework::ALL.to_vec(),
            scenarios: Scenario::ALL.to_vec(),
            runs: 5,
            timing: Timing {
                settle: Duration::from_secs(2),
                duration: Duration::from_secs(10),
            },
            placement: Placement::Headless,
            frame_rate: FrameRate::Display,
            build: true,
        };
        let mut arguments = arguments.peekable();
        while let Some(flag) = arguments.next() {
            match flag.as_str() {
                "--no-build" => {
                    options.build = false;
                    continue;
                }
                "--uncapped" => {
                    options.frame_rate = FrameRate::Uncapped;
                    continue;
                }
                _ => {}
            }
            let value = arguments
                .next()
                .with_context(|| format!("{flag} needs a value"))?;
            match flag.as_str() {
                "--frameworks" => options.frameworks = list(&value)?,
                "--scenarios" => options.scenarios = list(&value)?,
                "--runs" => options.runs = value.parse().context("--runs takes a count")?,
                "--duration" => options.timing.duration = seconds(&value)?,
                "--settle" => options.timing.settle = seconds(&value)?,
                "--placement" => options.placement = value.parse()?,
                other => bail!(
                    "unknown argument {other:?}; expected --frameworks, --scenarios, --runs, --duration, --settle, --placement, --uncapped or --no-build"
                ),
            }
        }
        Ok(options)
    }
}

fn list<T: FromStr<Err = anyhow::Error>>(value: &str) -> Result<Vec<T>> {
    value.split(',').map(str::parse).collect()
}

fn seconds(value: &str) -> Result<Duration> {
    Ok(Duration::from_secs_f64(
        value.parse().context("expected a number of seconds")?,
    ))
}

/// Counts long frames against the compositor's pace, measured from the apps that use frame
/// callbacks and shared by those that do not; returns that pace.
fn count_long_frames(measurements: &mut [Measurement]) -> Option<f64> {
    let paces: Vec<f64> = measurements
        .iter()
        .filter_map(|run| run.compositor_hz)
        .collect();
    let session_pace =
        median(&paces).or_else(|| measurements.iter().find_map(|run| run.refresh_hz));
    for run in measurements.iter_mut() {
        let pace = run.compositor_hz.or(session_pace);
        if let (Some(motion), Some(pace)) = (run.motion.as_mut(), pace) {
            motion.count_long_frames(pace);
        }
    }
    session_pace
}

fn main() -> Result<()> {
    let options = Options::parse(std::env::args().skip(1))?;
    let paths = Paths::discover()?;
    let spec = Spec::load(&paths.spec())?;
    if options.build {
        for framework in &options.frameworks {
            eprintln!("building {}", framework.name());
            framework
                .build(&paths, options.frame_rate)
                .with_context(|| format!("building {}", framework.name()))?;
        }
    }
    let footprints = options
        .frameworks
        .iter()
        .filter_map(|framework| Some((*framework, Footprint::measure(*framework, &paths).ok()?)))
        .collect();
    let machine = Machine::describe();
    let bench = Bench {
        paths: &paths,
        spec: &spec,
        timing: options.timing,
        placement: options.placement,
        frame_rate: options.frame_rate,
    };
    let _keep_awake = match options.placement {
        Placement::Headless => KeepAwake::start(&paths, &spec)
            .inspect_err(|error| eprintln!("the screen may lock mid-run: {error:#}"))
            .ok(),
        Placement::Desktop => None,
    };
    let mut measurements = Vec::new();
    let mut failures = Vec::new();
    let mut stopped_early = false;
    'rounds: for run in 0..=options.runs {
        for scenario in &options.scenarios {
            for framework in &options.frameworks {
                let label = format!("{} {} run {run}", framework.name(), scenario.name());
                match bench.measure(*framework, *scenario, run) {
                    Ok(_) if run == 0 => eprintln!("{label}: warm-up"),
                    Ok(measurement) => {
                        eprintln!("{label}: startup {:.0} ms", measurement.startup_ms);
                        measurements.push(measurement);
                    }
                    Err(error) if error.is::<WindowNotShown>() => {
                        eprintln!(
                            "{label}: {error}; stopping, since every later run would fail too"
                        );
                        failures.push(Failure {
                            framework: *framework,
                            scenario: *scenario,
                            run,
                            error: error.to_string(),
                        });
                        stopped_early = true;
                        break 'rounds;
                    }
                    Err(error) => {
                        eprintln!("{label}: {error:#}");
                        failures.push(Failure {
                            framework: *framework,
                            scenario: *scenario,
                            run,
                            error: format!("{error:#}"),
                        });
                    }
                }
            }
        }
    }
    if stopped_early {
        bail!("stopped early, no results written: {WindowNotShown}");
    }
    let compositor_hz = count_long_frames(&mut measurements);
    let results = Results {
        machine,
        placement: options.placement,
        frame_rate: options.frame_rate,
        compositor_hz,
        spec,
        timing: options.timing,
        runs: options.runs,
        frameworks: options.frameworks,
        footprints,
        measurements,
        failures,
    };
    let written = results.write(paths.results.clone())?;
    println!("{}", results.markdown());
    eprintln!("wrote {}", written.display());
    Ok(())
}
