use view::AnyView;
use windowing::{Waker, wayland::Output};

use crate::AppConfig;

/// A shell app that shows its surfaces on some of the monitors, run by
/// [`run_surfaces`](crate::run_surfaces).
///
/// The runtime places surfaces again once a dispatch of events is in, when a monitor appeared,
/// changed or went in it, or when the app woke the loop through the [`Waker`]: first
/// [`outputs`](Surfaces::outputs) with every monitor, then [`placement`](Surfaces::placement) for
/// each. State that all surfaces share lives in this value and is cloned into each
/// [`build`](Surfaces::build).
pub trait Surfaces: 'static {
    /// Receives the waker once at start; calling it makes the runtime place surfaces again.
    fn attach(&mut self, _waker: Waker) {}

    /// Every monitor, in the order they were announced, before the placements that follow. An app
    /// that wants one surface for all monitors picks its output here, for example the focused one
    /// else the first.
    fn outputs(&mut self, _outputs: &[Output]) {}

    /// Whether `output` gets surfaces: `None` for none, `Some(key)` for surfaces that are built
    /// again whenever the key changes. It runs each time surfaces are placed, so keep it cheap.
    fn placement(&mut self, output: &Output) -> Option<u64>;

    /// The surfaces to show on `output`. The runtime puts each layer surface on `output`.
    fn build(&mut self, output: &Output) -> Vec<(AnyView, AppConfig)>;
}
