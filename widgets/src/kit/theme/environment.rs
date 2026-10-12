use reactive::Signal;
use view::Cx;

use super::definition::Theme;

/// Provides `theme` to the current scope and everything built inside it; setting the returned
/// signal restyles every kit widget below.
pub fn provide_theme(cx: &mut Cx, theme: Theme) -> Signal<Theme> {
    cx.provide(theme)
}

/// The nearest provided theme, or the default one provided on the current scope when there is none.
pub fn use_theme(cx: &mut Cx) -> Signal<Theme> {
    match cx.environment::<Theme>() {
        Some(theme) => theme,
        None => provide_theme(cx, Theme::DEFAULT),
    }
}
