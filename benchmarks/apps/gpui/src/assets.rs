use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};

pub const SQUARE: &str = "square.svg";

/// The SVGs the scenarios draw: the square the animation rotates (GPUI rotates SVGs, not divs)
/// and the settings window's Lucide icons.
const FILES: [(&str, &[u8]); 10] = [
    (SQUARE, include_bytes!("../assets/square.svg")),
    ("icons/wifi.svg", include_bytes!("../../web/icons/wifi.svg")),
    (
        "icons/bluetooth.svg",
        include_bytes!("../../web/icons/bluetooth.svg"),
    ),
    (
        "icons/globe.svg",
        include_bytes!("../../web/icons/globe.svg"),
    ),
    (
        "icons/monitor.svg",
        include_bytes!("../../web/icons/monitor.svg"),
    ),
    (
        "icons/volume-2.svg",
        include_bytes!("../../web/icons/volume-2.svg"),
    ),
    ("icons/bell.svg", include_bytes!("../../web/icons/bell.svg")),
    (
        "icons/battery-full.svg",
        include_bytes!("../../web/icons/battery-full.svg"),
    ),
    (
        "icons/palette.svg",
        include_bytes!("../../web/icons/palette.svg"),
    ),
    (
        "icons/chevron-right.svg",
        include_bytes!("../../web/icons/chevron-right.svg"),
    ),
];

pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(FILES
            .iter()
            .find(|(name, _)| *name == path)
            .map(|(_, bytes)| Cow::Borrowed(*bytes)))
    }

    fn list(&self, _path: &str) -> Result<Vec<SharedString>> {
        Ok(FILES.iter().map(|(name, _)| (*name).into()).collect())
    }
}
