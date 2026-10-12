//! CrownOS-styled widgets: one themed palette, configured through `crownconfig`, behind every control.

mod badge;
mod button;
mod calendar;
mod callout;
mod card;
mod choice_list;
mod context_menu;
mod divider;
mod font;
mod icon;
mod keybind_input;
mod layouts;
mod pane;
mod popup_menu;
mod preview;
mod previews;
mod progress;
mod quick_panel;
mod radio_group;
mod radio_mark;
mod search;
mod section;
mod segmented_bar;
mod select;
mod setting_row;
mod shadow_box;
mod slider;
mod spinner;
mod status;
mod step_bar;
mod text_input;
mod theme;
mod toggle;
mod toggle_group;
mod tone;

pub use badge::{BadgeKind, BadgeOptions, badge};
pub use button::{ButtonKind, ButtonOptions, ButtonSize, ButtonVariant, button, icon_button};
pub use calendar::{CalendarDate, CalendarKind, calendar};
pub use callout::{CalloutKind, CalloutOptions, callout};
pub use card::{CardKind, card};
pub use choice_list::{Choice, ChoiceListKind, choice_list};
pub use context_menu::{
    ContextMenuKind, MenuHeaderKind, MenuRowKind, MenuRowOptions, MenuSeparatorKind, context_menu,
    menu_header, menu_item, menu_item_disabled, menu_item_selected, menu_separator, menu_submenu,
    preview_menu,
};
pub use divider::{DividerKind, DividerOptions, divider};
pub use font::{INTER_FAMILY, TEXT_SIZE, inter_font};
pub use icon::{IconKind, icon, icon_colored, icon_muted, icons};
pub use keybind_input::{KeybindInputKind, KeybindInputOptions, keybind_input};
pub use layouts::{
    HERO_CARD_RADIUS, HERO_COLUMN_WIDTH, HeroStep, HeroWizardKind, SidebarCardKind,
    SidebarCollapseKind, SidebarItemKind, SidebarItemOptions, SidebarItemVariant, SidebarMenuKind,
    SidebarSectionKind, SidebarSeparatorKind, SlideTransitionKind, WizardKind, WizardPageKind,
    WizardRailKind, hero_pane, hero_wizard, sidebar, sidebar_brand, sidebar_collapse,
    sidebar_column, sidebar_group, sidebar_item, sidebar_section, sidebar_separator,
    sidebar_subitem, sidebar_with_header, slide_transition, wizard, wizard_footer, wizard_page,
    wizard_rail, wizard_scroll,
};
pub use pane::{PaneFill, PaneKind, PaneOptions, Radii, pane};
pub use popup_menu::{
    MenuEntry, MenuFlag, PopupMenuKind, PopupMenuOptions, PopupMenuPanelKind, PopupMenuRowKind,
    PopupMenuSeparatorKind, PopupMenuTriggerKind, menu_item as popup_menu_item,
    menu_separator as popup_menu_separator, popup_menu,
};
pub use preview::Preview;
pub use previews::previews;
pub use progress::{ProgressKind, progress};
pub use quick_panel::{
    PANEL_RADIUS, PANEL_WIDTH, PanelActionKind, PanelGlyph, PanelHeaderKind, PanelHeaderOptions,
    PanelItemKind, PanelItemOptions, PanelReadoutKind, PanelSectionKind, PanelSectionOptions,
    PanelSeparatorKind, PanelSliderKind, PanelSliderOptions, QuickPanelKind, panel_action,
    panel_fill, panel_header, panel_item, panel_readout, panel_rim, panel_section, panel_separator,
    panel_slider, quick_panel,
};
pub use radio_group::{RadioGroupKind, radio_group};
pub use search::{DEFAULT_PLACEHOLDER, SearchKind, SearchOptions, Suggestion, search, suggestion};
pub use section::{SectionKind, section};
pub use segmented_bar::{Segment, SegmentedBarKind, SegmentedBarOptions, segmented_bar};
pub use select::{SelectKind, SelectOptions, SelectPanelKind, SelectRowKind, select};
pub use setting_row::{SettingRowKind, setting_row};
pub use shadow_box::{SoftShadowKind, soft_shadow};
pub use slider::{SliderKind, SliderOptions, slider};
pub use spinner::{SpinnerKind, spinner};
pub use status::{StatusKind, StatusOptions, status};
pub use step_bar::{IntoStep, StepBarKind, step_bar};
pub use text_input::{ClearButton, FieldShape, TextInputKind, TextInputOptions, text_input};
pub use toggle::{ToggleKind, ToggleOptions, toggle};
pub use toggle_group::{ToggleGroupKind, ToggleGroupOptions, ToggleItem, toggle_group};
pub use tone::Tone;

pub use theme::{
    ControlColors, DEFAULT_ACCENT, GradientStops, MenuColors, PopoverColors, RadioColors,
    SidebarColors, StatusColors, SurfaceColors, TextColors, Theme, ThemeMode, accent_stops,
    provide_configured_theme, provide_theme, use_theme,
};
