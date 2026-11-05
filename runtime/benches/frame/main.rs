//! Frame benchmarks: seeded element trees at several sizes, one bench per kind of change.
//! `cargo bench -p runtime --bench frame -- [--filter NAME] [--families wide,deep,mixed]
//! [--sizes 256,1000,10000] [--iterations N] [--json PATH] [--baseline PATH]`, or `cargo xtask bench`.

mod fixture;
mod instructions;
mod measure;
mod options;
mod report;
mod scenario;
mod tree;

use anyhow::Result;

use crate::{measure::Record, options::Options, scenario::Scenario};

fn main() -> Result<()> {
    let options = Options::parse(std::env::args().skip(1))?;
    let mut records = Vec::new();
    println!("{}", Record::header());
    for &family in &options.families {
        for &size in &options.sizes {
            for scenario in Scenario::ALL
                .into_iter()
                .filter(|scenario| options.selects(*scenario))
            {
                let record = measure::run(scenario, family, size, options.iterations)?;
                println!("{}", record.row());
                records.push(record);
            }
        }
    }
    if let Some(baseline) = &options.baseline {
        print!("{}", report::compare(&records, baseline)?);
    }
    if let Some(path) = &options.json {
        report::write(&records, path)?;
    }
    Ok(())
}
