use geometry::Color;

use crate::kit::MenuColors;

/// Which kind of row a builder made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RowTone {
    Normal,
    Selected,
    Disabled,
    Submenu,
}

/// How a row presents itself right now; the colors it resolves are the palette's `menu` group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RowState {
    Idle,
    Highlighted,
    Disabled,
}

impl RowState {
    pub(super) fn label_color(self, menu: &MenuColors) -> Color {
        match self {
            Self::Idle => menu.text,
            Self::Highlighted => menu.selected_text,
            Self::Disabled => menu.disabled_text,
        }
    }

    pub(super) fn shortcut_color(self, menu: &MenuColors) -> Color {
        match self {
            Self::Idle => menu.shortcut_text,
            Self::Highlighted => menu.selected_text,
            Self::Disabled => menu.disabled_text,
        }
    }
}
