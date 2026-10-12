use view::Cx;

use super::flag::MenuFlag;

pub(super) type SelectCallback = Box<dyn FnMut(&mut Cx)>;

/// What a row looks like and whether a press on it does anything; the callback lives apart.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct RowSpec {
    pub(super) label: String,
    pub(super) separator: bool,
    pub(super) checked: Option<MenuFlag>,
    pub(super) destructive: bool,
    pub(super) disabled: MenuFlag,
}

/// One entry of a [`popup_menu`](super::popup_menu): a row with what to run when it is picked, or
/// a separator.
#[must_use = "entries do nothing unless handed to popup_menu"]
pub struct MenuEntry {
    pub(super) spec: RowSpec,
    pub(super) on_select: Option<SelectCallback>,
}

/// A row that runs `on_select` when it is picked. Exported by the kit as `popup_menu_item`,
/// because [`menu_item`](crate::kit::menu_item) already names the display-only row.
pub fn menu_item(label: impl Into<String>, on_select: impl FnMut(&mut Cx) + 'static) -> MenuEntry {
    MenuEntry {
        spec: RowSpec {
            label: label.into(),
            separator: false,
            checked: None,
            destructive: false,
            disabled: MenuFlag::Fixed(false),
        },
        on_select: Some(Box::new(on_select)),
    }
}

/// A hairline between two groups of rows. Exported by the kit as `popup_menu_separator`.
pub fn menu_separator() -> MenuEntry {
    MenuEntry {
        spec: RowSpec {
            label: String::new(),
            separator: true,
            checked: None,
            destructive: false,
            disabled: MenuFlag::Fixed(true),
        },
        on_select: None,
    }
}

impl MenuEntry {
    /// Draws a leading checkmark while `checked` holds: for a row that toggles something.
    pub fn checked(mut self, checked: impl Into<MenuFlag>) -> Self {
        self.spec.checked = Some(checked.into());
        self
    }

    /// Letters the row in the palette's danger color: for the one row that destroys something.
    pub fn destructive(mut self) -> Self {
        self.spec.destructive = true;
        self
    }

    /// Dims the row and ignores presses on it while `disabled` holds.
    pub fn disabled(mut self, disabled: impl Into<MenuFlag>) -> Self {
        self.spec.disabled = disabled.into();
        self
    }
}
