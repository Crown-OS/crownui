use std::time::Instant;

use anyhow::Result;
use runtime::RenderStats;
use serde::{Deserialize, Serialize};
use view::UiCounters;

use crate::{fixture::Fixture, instructions::Instructions, scenario::Scenario, tree::Family};

const WARMUP_STEPS: usize = 8;

/// One scenario's measurements on one tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Record {
    pub(crate) scenario: String,
    pub(crate) family: String,
    pub(crate) size: usize,
    pub(crate) nodes: usize,
    pub(crate) median_ns: u64,
    pub(crate) min_ns: u64,
    pub(crate) instructions: Option<u64>,
    pub(crate) layout_passes: u32,
    pub(crate) layout_visits: u32,
    pub(crate) op_visits: u32,
    pub(crate) ops: u32,
    pub(crate) hit_tests: u32,
    pub(crate) hit_visits: u32,
    pub(crate) semantics_rebuilds: u32,
    pub(crate) recorded: usize,
    pub(crate) composite_ops: usize,
}

impl Record {
    pub(crate) fn header() -> String {
        format!(
            "{:<18} {:<5} {:>6} {:>6} {:>10} {:>9}  {:>3} {:>7} {:>7} {:>5} {:>4} {:>6} {:>3} {:>5} {:>6}",
            "scenario",
            "tree",
            "size",
            "nodes",
            "median_us",
            "kinstr",
            "lay",
            "lvisit",
            "ovisit",
            "ops",
            "hits",
            "hvisit",
            "sem",
            "rec",
            "cops"
        )
    }

    pub(crate) fn row(&self) -> String {
        let instructions = self
            .instructions
            .map_or_else(|| "n/a".to_owned(), |count| (count / 1_000).to_string());
        format!(
            "{:<18} {:<5} {:>6} {:>6} {:>10.1} {:>9}  {:>3} {:>7} {:>7} {:>5} {:>4} {:>6} {:>3} {:>5} {:>6}",
            self.scenario,
            self.family,
            self.size,
            self.nodes,
            self.median_ns as f64 / 1_000.0,
            instructions,
            self.layout_passes,
            self.layout_visits,
            self.op_visits,
            self.ops,
            self.hit_tests,
            self.hit_visits,
            self.semantics_rebuilds,
            self.recorded,
            self.composite_ops
        )
    }

    /// The counters, named, for change reports.
    pub(crate) fn counters(&self) -> [(&'static str, u64); 9] {
        [
            ("layout_passes", u64::from(self.layout_passes)),
            ("layout_visits", u64::from(self.layout_visits)),
            ("op_visits", u64::from(self.op_visits)),
            ("ops", u64::from(self.ops)),
            ("hit_tests", u64::from(self.hit_tests)),
            ("hit_visits", u64::from(self.hit_visits)),
            ("semantics_rebuilds", u64::from(self.semantics_rebuilds)),
            ("recorded", self.recorded as u64),
            ("composite_ops", self.composite_ops as u64),
        ]
    }
}

/// Builds the tree, warms up, then times `iterations` steps of `scenario`.
pub(crate) fn run(
    scenario: Scenario,
    family: Family,
    size: usize,
    iterations: usize,
) -> Result<Record> {
    let mut fixture = Fixture::new(family, size)?;
    for _ in 0..WARMUP_STEPS {
        scenario.step(&mut fixture)?;
    }
    let mut counter = Instructions::open();
    let mut times = Vec::with_capacity(iterations);
    let mut instructions = Vec::with_capacity(iterations);
    let mut last = (UiCounters::default(), RenderStats::default());
    for _ in 0..iterations.max(1) {
        counter.start();
        let started = Instant::now();
        last = scenario.step(&mut fixture)?;
        let elapsed = started.elapsed();
        if let Some(count) = counter.stop() {
            instructions.push(count);
        }
        times.push(u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX));
    }
    times.sort_unstable();
    instructions.sort_unstable();
    let (counters, stats) = last;
    Ok(Record {
        scenario: scenario.name().to_owned(),
        family: family.name().to_owned(),
        size,
        nodes: fixture.app.render_tree().len(),
        median_ns: median(&times).unwrap_or(0),
        min_ns: times.first().copied().unwrap_or(0),
        instructions: median(&instructions),
        layout_passes: counters.layout_passes,
        layout_visits: counters.layout_visits,
        op_visits: counters.op_visits,
        ops: counters.ops,
        hit_tests: counters.hit_tests,
        hit_visits: counters.hit_visits,
        semantics_rebuilds: counters.semantics_rebuilds,
        recorded: stats.recorded,
        composite_ops: stats.composite_ops,
    })
}

fn median(sorted: &[u64]) -> Option<u64> {
    sorted.get(sorted.len() / 2).copied()
}
