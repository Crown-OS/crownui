use std::collections::BTreeMap;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::{
    report::{Report, Step},
    workspace,
};

#[derive(Deserialize)]
struct LayerFile {
    layers: toml::Table,
    #[serde(default)]
    allow: BTreeMap<String, Vec<String>>,
}

/// Fails when a crate depends on a crate in the same or a higher layer without an allowance.
pub(crate) fn run() -> Result<Report> {
    let root = workspace::root()?;
    let text = std::fs::read_to_string(root.join("layers.toml")).context("reading layers.toml")?;
    let file: LayerFile = toml::from_str(&text).context("parsing layers.toml")?;
    let mut rank: BTreeMap<String, usize> = BTreeMap::new();
    for (index, (_, crates)) in file.layers.iter().enumerate() {
        for name in crates
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(toml::Value::as_str)
        {
            rank.insert(name.to_owned(), index);
        }
    }
    let mut steps = Vec::new();
    for member in workspace::members()? {
        let Some(&own) = rank.get(&member.name) else {
            steps.push(Step {
                name: format!("{} is not listed in layers.toml", member.name),
                ok: false,
                detail: String::new(),
            });
            continue;
        };
        let allowed = file.allow.get(&member.name).cloned().unwrap_or_default();
        let violations: Vec<String> = member
            .dependencies
            .iter()
            .filter(|dependency| {
                let theirs = rank.get(*dependency).copied().unwrap_or(usize::MAX);
                theirs >= own && !allowed.contains(dependency)
            })
            .cloned()
            .collect();
        steps.push(Step {
            name: format!("{} ({} deps)", member.name, member.dependencies.len()),
            ok: violations.is_empty(),
            detail: if violations.is_empty() {
                String::new()
            } else {
                format!("depends on same-or-higher layer: {}", violations.join(", "))
            },
        });
    }
    Ok(Report::new("layers", steps))
}
