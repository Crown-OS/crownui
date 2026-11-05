use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use crate::{scenario::Scenario, tree::Family};

const DEFAULT_SIZES: [usize; 3] = [256, 1_000, 10_000];
const DEFAULT_ITERATIONS: usize = 200;

pub(crate) struct Options {
    pub(crate) filter: Option<String>,
    pub(crate) families: Vec<Family>,
    pub(crate) sizes: Vec<usize>,
    pub(crate) iterations: usize,
    pub(crate) json: Option<PathBuf>,
    pub(crate) baseline: Option<PathBuf>,
}

impl Options {
    pub(crate) fn parse(arguments: impl Iterator<Item = String>) -> Result<Self> {
        let mut options = Self {
            filter: None,
            families: Family::ALL.to_vec(),
            sizes: DEFAULT_SIZES.to_vec(),
            iterations: DEFAULT_ITERATIONS,
            json: None,
            baseline: None,
        };
        let mut arguments = arguments;
        while let Some(argument) = arguments.next() {
            if argument == "--bench" {
                continue;
            }
            let value = arguments
                .next()
                .with_context(|| format!("{argument} needs a value"))?;
            match argument.as_str() {
                "--filter" => options.filter = Some(value),
                "--families" => {
                    options.families =
                        value.split(',').map(Family::parse).collect::<Result<_>>()?;
                }
                "--sizes" => {
                    options.sizes = value
                        .split(',')
                        .map(|size| {
                            size.trim()
                                .parse()
                                .with_context(|| format!("invalid size {size:?}"))
                        })
                        .collect::<Result<_>>()?;
                }
                "--iterations" => {
                    options.iterations = value.parse().context("invalid iteration count")?;
                }
                "--json" => options.json = Some(PathBuf::from(value)),
                "--baseline" => options.baseline = Some(PathBuf::from(value)),
                other => bail!("unknown argument {other:?}"),
            }
        }
        Ok(options)
    }

    pub(crate) fn selects(&self, scenario: Scenario) -> bool {
        self.filter
            .as_deref()
            .is_none_or(|filter| scenario.name().contains(filter))
    }
}
