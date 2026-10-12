use geometry::Color;

use crate::kit::MenuColors;

/// How a row presents itself; the colors it resolves are the palette's `menu` group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RowTone {
    Normal,
    Selected,
    Disabled,
    Submenu,
}

impl RowTone {
    pub(super) fn label_color(self, menu: &MenuColors) -> Color {
        match self {
            Self::Normal | Self::Submenu => menu.text,
            Self::Selected => menu.selected_text,
            Self::Disabled => menu.disabled_text,
        }
    }

    pub(super) fn shortcut_color(self, menu: &MenuColors) -> Color {
        match self {
            Self::Normal | Self::Submenu => menu.shortcut_text,
            Self::Selected => menu.selected_text,
            Self::Disabled => menu.disabled_text,
        }
    }
}
