use serde::Serialize;

/// Whether apps draw in step with the display or as fast as they can.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FrameRate {
    /// Every toolkit's default: frames paced by the compositor.
    Display,
    /// Frame pacing turned off where the toolkit allows it, to see how many frames it can draw:
    /// CrownUI with `FramePacing::Unpaced`, GPUI patched to skip frame callbacks. Qt Quick,
    /// Chromium and WebKitGTK stayed at the compositor's pace with every switch tried (a swap
    /// interval of 0, Qt's basic render loop, `--disable-frame-rate-limit --disable-gpu-vsync`),
    /// so they run as usual.
    Uncapped,
}

impl FrameRate {
    pub fn is_uncapped(self) -> bool {
        self == Self::Uncapped
    }
}
