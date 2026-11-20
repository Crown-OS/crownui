use view::AnyView;
use windowing::{Waker, wayland::Output};

use crate::AppConfig;

/// A shell app that shows its surfaces on some of the monitors, run by
/// [`run_surfaces`](crate::run_surfaces).
///
/// The runtime asks for a placement for every monitor whenever one appears, changes or goes,
/// and whenever the app wakes the loop through the [`Waker`]. State that all surfaces share
/// lives in this value and is cloned into each [`build`](Surfaces::build).
pub trait Surfaces: 'static {
    /// Receives the waker once at start; calling it makes the runtime ask for placements again.
    fn attach(&mut self, _waker: Waker) {}

    /// Whether `output` gets surfaces: `None` for none, `Some(key)` for surfaces that are built
    /// again whenever the key changes. It runs on every wake of the loop, so keep it cheap.
    fn placement(&mut self, output: &Output) -> Option<u64>;

    /// The surfaces to show on `output`. The runtime puts each layer surface on `output`.
    fn build(&mut self, output: &Output) -> Vec<(AnyView, AppConfig)>;
}
