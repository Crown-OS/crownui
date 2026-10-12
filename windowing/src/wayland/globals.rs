use sctk::{
    data_device_manager::DataDeviceManagerState,
    reexports::{
        client::{QueueHandle, globals::GlobalList},
        protocols::wp::{
            idle_inhibit::zv1::client::zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1,
            pointer_gestures::zv1::client::zwp_pointer_gestures_v1::ZwpPointerGesturesV1,
            presentation_time::client::wp_presentation::WpPresentation,
            text_input::zv3::client::zwp_text_input_manager_v3::ZwpTextInputManagerV3,
        },
    },
    shell::wlr_layer::LayerShell,
};

use crate::wayland::{
    color_management::ColorManagement, event_loop::WaylandPlatform, util::no_events::NoEvents,
};

/// Optional upstream globals; each one missing only switches its feature off.
pub(crate) struct StandardGlobals {
    pub(crate) presentation: Option<WpPresentation>,
    pub(crate) pointer_gestures: Option<ZwpPointerGesturesV1>,
    pub(crate) text_input: Option<ZwpTextInputManagerV3>,
    pub(crate) data_device: Option<DataDeviceManagerState>,
    pub(crate) color: Option<ColorManagement>,
    pub(crate) layer_shell: Option<LayerShell>,
    pub(crate) idle_inhibit: Option<ZwpIdleInhibitManagerV1>,
}

impl StandardGlobals {
    pub(crate) fn bind(globals: &GlobalList, qh: &QueueHandle<WaylandPlatform>) -> Self {
        Self {
            presentation: globals.bind(qh, 1..=2, NoEvents).ok(),
            pointer_gestures: globals.bind(qh, 1..=3, NoEvents).ok(),
            text_input: globals.bind(qh, 1..=1, NoEvents).ok(),
            data_device: DataDeviceManagerState::bind(globals, qh).ok(),
            color: ColorManagement::bind(globals, qh),
            layer_shell: LayerShell::bind(globals, qh).ok(),
            idle_inhibit: globals.bind(qh, 1..=1, NoEvents).ok(),
        }
    }
}
