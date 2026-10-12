mod capture;
mod injector;

pub use capture::{CapturedInput, InputCapture, InputCaptureEvent, InputCaptureId};
pub use crownos_protocols::input::v1::client::crownos_input_capture_v1::Edge as CaptureEdges;
pub use injector::{InjectedInput, InputInjector, InputInjectorId};

/// Whether a captured or injected button or key went down or up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ElementState {
    Pressed,
    Released,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScrollAxis {
    Vertical,
    Horizontal,
}
