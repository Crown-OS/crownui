/// When the render thread draws a window's next frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FramePacing {
    /// In step with the display: each frame waits for the platform's frame callback.
    #[default]
    Display,
    /// As soon as the previous frame is submitted, never waiting for the display: frames the
    /// display cannot show are drawn and dropped. For measuring throughput, not for shipping.
    Unpaced,
}
