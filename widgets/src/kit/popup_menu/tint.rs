use geometry::Color;

use crate::kit::Theme;

pub(super) fn row_content_color(
    theme: &Theme,
    highlighted: bool,
    disabled: bool,
    destructive: bool,
) -> Color {
    match (highlighted, disabled, destructive) {
        (true, _, _) => theme.menu.selected_text,
        (false, true, _) => theme.menu.disabled_text,
        (false, false, true) => theme.status.danger,
        (false, false, false) => theme.menu.text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_highlight_overrides_every_resting_tone() {
        let theme = Theme::dark();
        for (disabled, destructive) in [(false, false), (true, false), (false, true)] {
            assert_eq!(
                row_content_color(&theme, true, disabled, destructive),
                theme.menu.selected_text
            );
        }
    }

    #[test]
    fn resting_rows_are_lettered_by_kind() {
        let theme = Theme::light();
        assert_eq!(
            row_content_color(&theme, false, false, false),
            theme.menu.text
        );
        assert_eq!(
            row_content_color(&theme, false, true, false),
            theme.menu.disabled_text
        );
        assert_eq!(
            row_content_color(&theme, false, false, true),
            theme.status.danger
        );
    }
}
