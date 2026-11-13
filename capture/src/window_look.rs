use std::cell::Cell;

use geometry::{Color, CornerShape};
use windowing::Material;

/// How the real runner dresses a window: what `AppConfig` gives it besides the view.
#[derive(Debug, Clone, Default)]
pub(crate) struct WindowLook {
    pub(crate) clear_color: Option<Color>,
    pub(crate) corner_shape: CornerShape,
    pub(crate) material: Option<Material>,
}

thread_local! {
    static LOOK: Cell<Option<WindowLook>> = const { Cell::new(None) };
    static FONTS: Cell<Vec<(Vec<u8>, Option<String>)>> = const { Cell::new(Vec::new()) };
}

/// Hands the next renderer and text system created on this thread the app's look and fonts.
pub(crate) fn prepare(look: WindowLook, fonts: Vec<(Vec<u8>, Option<String>)>) {
    LOOK.with(|cell| cell.set(Some(look)));
    FONTS.with(|cell| cell.set(fonts));
}

pub(crate) fn take_look() -> WindowLook {
    LOOK.with(Cell::take).unwrap_or_default()
}

pub(crate) fn take_fonts() -> Vec<(Vec<u8>, Option<String>)> {
    FONTS.with(Cell::take)
}
