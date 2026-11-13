use reactive::Runtime;
use semantics::Role;
use view::{Cx, Element, Kind, NodeId, Semantic, Styled, bind};

use super::{
    metrics::*,
    tone::{RowState, RowTone},
};
use crate::{
    kit::{INTER_FAMILY, icon::tinted_icon, icons, use_theme},
    spacer::spacer,
    stack::vstack,
    text::text,
    util::{anchored_popup::instant, glass::bind_capsule_highlight, interaction::Interaction},
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
        let disabled = cx.observe_disabled(node);
        let interaction = Interaction::track(cx, node);
        let state = move |runtime: &mut Runtime| {
            if disabled.get(runtime) {
                RowState::Disabled
            } else if tone == RowTone::Selected || interaction.is_hovered(runtime) {
                RowState::Highlighted
            } else {
                RowState::Idle
            }
        };
        let label_color = move |runtime: &mut Runtime| {
            let state = state(runtime);
            theme.with(runtime, |theme| state.label_color(&theme.menu))
        };
        let shortcut_color = move |runtime: &mut Runtime| {
            let state = state(runtime);
            theme.with(runtime, |theme| state.shortcut_color(&theme.menu))
        };

        cx.set_role(node, Role::MenuItem);
        bind_capsule_highlight(cx, node, theme, move |runtime| {
            state(runtime) == RowState::Highlighted
        });

        let leading = self.icon.map(|body| {
            vstack(tinted_icon(body, ICON_STROKE, bind(label_color)).size(ICON_SIZE))
                .w(ICON_SLOT_WIDTH)
                .shrink(0.0)
        });
        let label = text(self.label)
            .text_size(LABEL_FONT_SIZE)
            .text_color(bind(label_color));
        let shortcut = (tone != RowTone::Submenu && !self.shortcut.is_empty()).then(|| {
            text(self.shortcut)
                .text_size(SHORTCUT_FONT_SIZE)
                .text_color(bind(shortcut_color))
                .ml(TRAILING_GAP)
        });
        let chevron = (tone == RowTone::Submenu).then(|| {
            tinted_icon(icons::CHEVRON_RIGHT, CHEVRON_STROKE, bind(label_color))
                .size(CHEVRON_SIZE)
                .ml(TRAILING_GAP)
                .mr(-CHEVRON_SIDE_BEARING)
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
    .disabled(tone == RowTone::Disabled)
    .flex_row()
    .items_center()
    .self_stretch()
    .h(ROW_HEIGHT)
    .px(ROW_H_PADDING)
    .rounded(HIGHLIGHT_RADIUS)
    .border(HIGHLIGHT_BORDER_WIDTH)
    .shrink(0.0)
    .font_family(INTER_FAMILY)
    .transition(instant())
}

/// A standard row: icon slot, label and a trailing shortcut hint. It lights up in the accent
/// capsule under the pointer.
pub fn menu_item(label: impl Into<String>) -> Element<MenuRowKind> {
    row(label, RowTone::Normal)
}

/// A row held in the highlighted state: an accent capsule lettered in `selected_text`.
pub fn menu_item_selected(label: impl Into<String>) -> Element<MenuRowKind> {
    row(label, RowTone::Selected)
}

/// A dimmed row that does nothing.
pub fn menu_item_disabled(label: impl Into<String>) -> Element<MenuRowKind> {
    row(label, RowTone::Disabled)
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
