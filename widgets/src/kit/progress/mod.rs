mod kind;

pub use kind::ProgressKind;

use view::{Element, IntoProp, Styled};

use crate::util::theme_fade::theme_fade;

use kind::{DEFAULT_WIDTH, HEIGHT};

/// A determinate bar `fraction` of the way along (clamped to `0..=1`); resize with `.w(..)`.
pub fn progress(fraction: impl IntoProp<f32>) -> Element<ProgressKind> {
    Element::new(ProgressKind {
        fraction: fraction.into_prop(),
    })
    .relative()
    .self_start()
    .shrink(0.0)
    .w(DEFAULT_WIDTH)
    .h(HEIGHT)
    .rounded(HEIGHT / 2.0)
    .transition(theme_fade())
}
