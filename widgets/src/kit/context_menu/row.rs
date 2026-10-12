use semantics::Role;
use view::{Cx, Element, Kind, NodeId, Semantic, StyleProp, Styled};

use super::{metrics::*, tone::RowTone};
use crate::{
    kit::{INTER_FAMILY, icon_colored, icons, use_theme},
    spacer::spacer,
    stack::vstack,
    text::text,
    util::themed::{bind_themed_style, themed},
};

/// The kind behind [`menu_item`](super::menu_item) and its sibling rows.
pub struct MenuRowKind {
    label: String,
    tone: RowTone,
    icon: Option<&'static str>,
    shortcut: String,
}

impl Kind for MenuRowKind {
    const NAME: &'static str = "kit_menu_row";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let tone = self.tone;
        cx.set_role(node, Role::MenuItem);
        if tone == RowTone::Selected {
            bind_themed_style(cx, node, theme, |theme| {
                StyleProp::Background(Some(theme.menu.selected_bg))
            });
        }

        let leading = vstack(self.icon.map(|body| {
            icon_colored(body, ICON_STROKE, move |theme| {
                tone.label_color(&theme.menu)
            })
        }))
        .w(ICON_SLOT_WIDTH)
        .h(ICON_SIZE)
        .shrink(0.0);
        let label = text(self.label)
            .text_size(LABEL_FONT_SIZE)
            .text_color(themed(theme, move |theme| tone.label_color(&theme.menu)));
        let shortcut = (tone != RowTone::Submenu && !self.shortcut.is_empty()).then(|| {
            text(self.shortcut)
                .text_size(SHORTCUT_FONT_SIZE)
                .text_color(themed(theme, move |theme| tone.shortcut_color(&theme.menu)))
        });
        let chevron = (tone == RowTone::Submenu).then(|| {
            vstack(
                icon_colored(icons::CHEVRON_RIGHT, ICON_STROKE, move |theme| {
                    tone.label_color(&theme.menu)
                })
                .size(CHEVRON_SIZE),
            )
            .w(CHEVRON_BOX)
            .h(CHEVRON_BOX)
            .shrink(0.0)
        });
        cx.set_children(node, (leading, label, spacer(), shortcut, chevron));
    }
}

fn row(label: impl Into<String>, tone: RowTone) -> Element<MenuRowKind> {
    let label = label.into();
    Element::new(MenuRowKind {
        label: label.clone(),
        tone,
        icon: None,
        shortcut: String::new(),
    })
    .label(label)
    .flex_row()
    .items_center()
    .self_stretch()
    .h(ROW_HEIGHT)
    .px(ROW_H_PADDING)
    .rounded(if tone == RowTone::Selected {
        SELECTED_RADIUS
    } else {
        0.0
    })
    .shrink(0.0)
    .font_family(INTER_FAMILY)
}

/// A standard row: icon slot, label and a trailing shortcut hint.
pub fn menu_item(label: impl Into<String>) -> Element<MenuRowKind> {
    row(label, RowTone::Normal)
}

/// The highlighted row: filled with the menu's selection color and lettered in `selected_text`.
pub fn menu_item_selected(label: impl Into<String>) -> Element<MenuRowKind> {
    row(label, RowTone::Selected)
}

/// A dimmed row that does nothing.
pub fn menu_item_disabled(label: impl Into<String>) -> Element<MenuRowKind> {
    row(label, RowTone::Disabled).disabled(true)
}

/// A row with a trailing chevron in place of a shortcut.
pub fn menu_submenu(label: impl Into<String>) -> Element<MenuRowKind> {
    row(label, RowTone::Submenu)
}

/// The options of a menu row, chained before its modifiers.
pub trait MenuRowOptions: Sized {
    /// A Lucide icon from [`icons`](crate::kit::icons) in the row's leading slot.
    fn leading_icon(self, body: &'static str) -> Self;

    /// The keyboard shortcut hint at the row's trailing edge.
    fn shortcut_hint(self, hint: impl Into<String>) -> Self;
}

impl MenuRowOptions for Element<MenuRowKind> {
    fn leading_icon(self, body: &'static str) -> Self {
        self.configure(|kind| kind.icon = Some(body))
    }

    fn shortcut_hint(self, hint: impl Into<String>) -> Self {
        let hint = hint.into();
        self.description(hint.clone())
            .configure(|kind| kind.shortcut = hint)
    }
}
