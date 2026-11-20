use runtime::RunError;

use super::{Surfaces, surface_runner::SurfaceRunner};
use crate::run::run_event_loop;

/// Runs a shell app that keeps its surfaces on the monitors [`Surfaces::placement`] picks:
/// surfaces open when a monitor appears or its key changes and close when it goes, and the app
/// keeps running with no surface at all. Same threading rule as [`run`](crate::run).
pub fn run_surfaces(app: impl Surfaces) -> Result<(), RunError> {
    let mut runner = SurfaceRunner::new(app);
    run_event_loop(&mut runner)?;
    match runner.take_failure() {
        Some(failure) => Err(failure),
        None => Ok(()),
    }
}
