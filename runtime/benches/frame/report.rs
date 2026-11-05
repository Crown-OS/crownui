use std::{collections::BTreeMap, fmt::Write, fs, path::Path};

use anyhow::{Context, Result};

use crate::measure::Record;

pub(crate) fn write(records: &[Record], path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(records)?)
        .with_context(|| format!("writing {}", path.display()))
}

/// A table of how `records` moved against the baseline at `path`: time and instruction deltas,
/// and every counter that changed.
pub(crate) fn compare(records: &[Record], path: &Path) -> Result<String> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let baseline: Vec<Record> = serde_json::from_str(&text)?;
    let previous: BTreeMap<(&str, &str, usize), &Record> = baseline
        .iter()
        .map(|record| {
            (
                (
                    record.scenario.as_str(),
                    record.family.as_str(),
                    record.size,
                ),
                record,
            )
        })
        .collect();
    let mut out = String::new();
    writeln!(
        out,
        "\nagainst {}:\n{:<18} {:<5} {:>6} {:>9} {:>9}  counters",
        path.display(),
        "scenario",
        "tree",
        "size",
        "time",
        "instr"
    )?;
    for record in records {
        let key = (
            record.scenario.as_str(),
            record.family.as_str(),
            record.size,
        );
        let Some(before) = previous.get(&key) else {
            continue;
        };
        let instructions = match (before.instructions, record.instructions) {
            (Some(before), Some(after)) => format!("{:>+8.1}%", delta(before, after)),
            _ => "      n/a".to_owned(),
        };
        let changes: Vec<String> = before
            .counters()
            .into_iter()
            .zip(record.counters())
            .filter(|((_, before), (_, after))| before != after)
            .map(|((name, before), (_, after))| format!("{name} {before}>{after}"))
            .collect();
        writeln!(
            out,
            "{:<18} {:<5} {:>6} {:>+8.1}% {instructions}  {}",
            record.scenario,
            record.family,
            record.size,
            delta(before.median_ns, record.median_ns),
            changes.join(" ")
        )?;
    }
    Ok(out)
}

fn delta(before: u64, after: u64) -> f64 {
    if before == 0 {
        return 0.0;
    }
    (after as f64 - before as f64) / before as f64 * 100.0
}
