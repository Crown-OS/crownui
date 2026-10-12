mod badge;
mod button;
mod calendar;
mod callout;
mod card;
mod choice_list;
mod context_menu;
mod divider;
mod hero_wizard;
mod keybind_input;
mod pane;
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
mod slide_transition;
mod slider;
mod spinner;
mod status;
mod step_bar;
mod text_input;
mod toggle;
mod toggle_group;
mod wizard;

use super::preview::Preview;

/// One preview per kit widget, in catalog order.
pub fn previews() -> Vec<Preview> {
    [
        button::previews(),
        status::previews(),
        badge::previews(),
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
        slide_transition::previews(),
        wizard::previews(),
    ]
    .concat()
}
