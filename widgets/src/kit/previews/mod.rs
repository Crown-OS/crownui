mod badge;
mod button;
mod calendar;
mod callout;
mod card;
mod choice_list;
mod context_menu;
mod context_menu_at;
mod dialog;
mod divider;
mod hero_wizard;
mod keybind_input;
mod pane;
mod path_bar;
mod popup_menu;
mod progress;
mod quick_panel;
mod radio_group;
mod search;
mod section;
mod segmented_bar;
mod select;
mod setting_row;
mod shadow_box;
mod sidebar;
mod sidebar_item;
mod slide_transition;
mod slider;
mod spinner;
mod status;
mod step_bar;
mod tag_chip;
mod text_input;
mod toast;
mod toggle;
mod toggle_group;
mod tooltip;
mod wizard;

use super::preview::Preview;

/// One preview per kit widget, in catalog order.
pub fn previews() -> Vec<Preview> {
    [
        button::previews(),
        status::previews(),
        badge::previews(),
        tag_chip::previews(),
        callout::previews(),
        card::previews(),
        divider::previews(),
        shadow_box::previews(),
        spinner::previews(),
        pane::previews(),
        text_input::previews(),
        search::previews(),
        keybind_input::previews(),
        context_menu::previews(),
        context_menu_at::previews(),
        dialog::previews(),
        tooltip::previews(),
        toast::previews(),
        popup_menu::previews(),
        select::previews(),
        hero_wizard::previews(),
        progress::previews(),
        step_bar::previews(),
        segmented_bar::previews(),
        section::previews(),
        setting_row::previews(),
        choice_list::previews(),
        radio_group::previews(),
        slider::previews(),
        quick_panel::previews(),
        calendar::previews(),
        toggle::previews(),
        toggle_group::previews(),
        sidebar::previews(),
        sidebar_item::previews(),
        path_bar::previews(),
        slide_transition::previews(),
        wizard::previews(),
    ]
    .concat()
}
