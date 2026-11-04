//! CrownOS-styled widgets: one themed palette, configured through `crownconfig`, behind every control.

mod badge;
mod button;
mod calendar;
mod callout;
mod card;
mod choice_list;
mod context_menu;
mod dialog;
mod divider;
mod font;
mod icon;
mod keybind_input;
mod layouts;
mod level_meter;
#[cfg(feature = "audio")]
mod microphone;
mod pane;
mod path_bar;
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
mod tag_chip;
mod text_input;
mod theme;
mod title_bar;
mod toast;
mod toggle;
mod toggle_group;
mod tone;
mod tooltip;
mod waveform;
mod window_radius;

pub use badge::{BadgeKind, BadgeOptions, badge};
pub use button::{
    ButtonKind, ButtonOptions, ButtonShape, ButtonSize, ButtonVariant, button, icon_button,
};
pub use calendar::{CalendarDate, CalendarKind, calendar};
pub use callout::{CalloutKind, CalloutOptions, callout};
pub use card::{BORDER_WIDTH as CARD_BORDER_WIDTH, RADIUS as CARD_RADIUS};
pub use card::{CardKind, card};
pub use choice_list::{Choice, ChoiceListKind, choice_list};
pub use context_menu::{
    ContextMenuAtKind, ContextMenuKind, ContextMenuLayerKind, MenuHeaderKind, MenuRowKind,
    MenuRowOptions, MenuSeparatorKind, context_menu, context_menu_at, menu_header, menu_item,
    menu_item_disabled, menu_item_selected, menu_separator, menu_submenu, preview_menu,
};
pub use dialog::{DialogKind, DialogOptions, DialogPanelKind, DialogSheetKind, dialog};
pub use divider::{DividerKind, DividerOptions, divider};
pub use font::{INTER_FAMILY, TEXT_SIZE, inter_font};
pub use icon::{IconKind, icon, icon_colored, icon_muted, icons};
pub use keybind_input::{KeybindInputKind, KeybindInputOptions, keybind_input};
pub use layouts::{
    HERO_CARD_RADIUS, HERO_COLUMN_WIDTH, HeroStep, HeroWizardKind, SidebarCardKind,
    SidebarCollapseKind, SidebarItemKind, SidebarItemOptions, SidebarItemVariant, SidebarMenuKind,
    SidebarSectionKind, SidebarSeparatorKind, SidebarShellKind, SlideTransitionKind, WizardKind,
    WizardPageKind, WizardRailKind, hero_pane, hero_wizard, sidebar, sidebar_brand,
    sidebar_collapse, sidebar_column, sidebar_group, sidebar_item, sidebar_search, sidebar_section,
    sidebar_separator, sidebar_subitem, sidebar_with_header, slide_transition, wizard,
    wizard_footer, wizard_page, wizard_rail, wizard_scroll,
};
pub use level_meter::{LevelMeterKind, LevelMeterOptions, level_meter};
#[cfg(feature = "audio")]
pub use microphone::{MicrophoneSignals, use_microphone};
pub use pane::{PaneFill, PaneKind, PaneOptions, Radii, pane};
pub use path_bar::{PathBarKind, PathBarOptions, PathSegment, path_bar};
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
pub use tag_chip::{TagChipKind, tag_chip};
pub use text_input::{ClearButton, FieldShape, TextInputKind, TextInputOptions, text_input};
pub use title_bar::{
    TitleBarSpacerKind, WindowControl, WindowControlKind, title_bar_spacer, window_controls,
};
pub use toast::{
    Toast, ToastHostKind, ToastKind, ToastOptions, Toasts, provide_toasts, toast, toast_host,
    use_toasts,
};
pub use toggle::{ToggleKind, ToggleOptions, toggle};
pub use toggle_group::{ToggleGroupKind, ToggleGroupOptions, ToggleItem, toggle_group};
pub use tone::Tone;
pub use tooltip::{TooltipBubbleKind, TooltipKind, tooltip};
pub use waveform::{
    LevelHistory, LiveWaveformKind, WaveformKind, WaveformOptions, WaveformPeaks, live_waveform,
    waveform,
};
pub use window_radius::{WindowRadius, provide_window_radius};

pub use theme::{
    ControlColors, DEFAULT_ACCENT, GradientStops, MenuColors, PopoverColors, RadioColors,
    SidebarColors, StatusColors, SurfaceColors, TextColors, Theme, ThemeMode, TitleBarColors,
    accent_stops, provide_configured_theme, provide_theme, use_theme,
};
