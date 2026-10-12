use super::{
    container::context_menu,
    header::menu_header,
    row::{MenuRowOptions, menu_item, menu_item_disabled, menu_item_selected, menu_submenu},
    separator::menu_separator,
};
use crate::kit::icons;
use view::View;

/// A ready-made menu matching the reference image.
pub fn preview_menu() -> impl View {
    context_menu((
        menu_item("Undo")
            .leading_icon(icons::UNDO)
            .shortcut_hint("⌘ Z"),
        menu_item("Redo")
            .leading_icon(icons::REDO)
            .shortcut_hint("⇧ ⌘ Z"),
        menu_separator(),
        menu_item("Paste")
            .leading_icon(icons::CLIPBOARD)
            .shortcut_hint("⌘ V"),
        menu_item_selected("Paste and Match Style")
            .leading_icon(icons::CLIPBOARD)
            .shortcut_hint("⌥ ⇧ ⌘ V"),
        menu_item("Delete")
            .leading_icon(icons::TRASH_2)
            .shortcut_hint("⌫"),
        menu_item("Select All")
            .leading_icon(icons::SQUARE_ARROW_UP)
            .shortcut_hint("⌘ A"),
        menu_item("Paste as Quotation").shortcut_hint("⇧ ⌘ V"),
        menu_item_disabled("Add Link").shortcut_hint("⌘ K"),
        menu_separator(),
        menu_submenu("Find").leading_icon(icons::FILES),
        menu_separator(),
        menu_header("Header"),
        menu_item("Dictation").shortcut_hint("D"),
        menu_item("Emoji")
            .leading_icon(icons::SMILE)
            .shortcut_hint("E"),
    ))
}
