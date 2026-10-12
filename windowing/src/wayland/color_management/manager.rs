use sctk::{
    dispatch2::Dispatch2,
    reexports::{
        client::{Connection, QueueHandle, globals::GlobalList, protocol::wl_surface::WlSurface},
        protocols::wp::color_management::v1::client::{
            wp_color_management_surface_v1::WpColorManagementSurfaceV1,
            wp_color_manager_v1::{self, Primaries, RenderIntent, WpColorManagerV1},
            wp_image_description_v1::{self, WpImageDescriptionV1},
        },
    },
};

use super::support::ColorSupport;
use crate::{
    WindowId,
    wayland::{event_loop::WaylandPlatform, util::no_events::NoEvents},
};

enum SrgbDescription {
    AwaitingSupport,
    Pending(WpImageDescriptionV1),
    Ready(WpImageDescriptionV1),
    Unavailable,
}

/// `wp_color_manager_v1` and the one sRGB image description every window is tagged with.
pub(crate) struct ColorManagement {
    manager: WpColorManagerV1,
    support: ColorSupport,
    srgb: SrgbDescription,
}

impl ColorManagement {
    pub(crate) fn bind(globals: &GlobalList, qh: &QueueHandle<WaylandPlatform>) -> Option<Self> {
        Some(Self {
            manager: globals.bind(qh, 1..=2, ColorManagerData).ok()?,
            support: ColorSupport::default(),
            srgb: SrgbDescription::AwaitingSupport,
        })
    }

    /// Tags `surface` as sRGB from its next commit; `None` until the description is ready.
    pub(crate) fn tag_srgb(
        &self,
        surface: &WlSurface,
        qh: &QueueHandle<WaylandPlatform>,
    ) -> Option<SurfaceColorTag> {
        let SrgbDescription::Ready(description) = &self.srgb else {
            return None;
        };
        let color_surface = self.manager.get_surface(surface, qh, NoEvents);
        color_surface.set_image_description(description, RenderIntent::Perceptual);
        Some(SurfaceColorTag(color_surface))
    }

    fn request_srgb_description(&mut self, qh: &QueueHandle<WaylandPlatform>) {
        let Some(transfer_function) = self.support.srgb_transfer_function() else {
            self.srgb = SrgbDescription::Unavailable;
            return;
        };
        let creator = self.manager.create_parametric_creator(qh, NoEvents);
        creator.set_tf_named(transfer_function);
        creator.set_primaries_named(Primaries::Srgb);
        self.srgb = SrgbDescription::Pending(creator.create(qh, ImageDescriptionData));
    }
}

impl Drop for ColorManagement {
    fn drop(&mut self) {
        if let SrgbDescription::Pending(description) | SrgbDescription::Ready(description) =
            &self.srgb
        {
            description.destroy();
        }
        self.manager.destroy();
    }
}

/// A window surface's color-management object; dropping it removes the tag.
pub(crate) struct SurfaceColorTag(WpColorManagementSurfaceV1);

impl Drop for SurfaceColorTag {
    fn drop(&mut self) {
        self.0.destroy();
    }
}

impl WaylandPlatform {
    /// Tags the window's surface as sRGB through `wp_color_management_v1`, from its next
    /// commit, or as soon as the compositor has built the description. Only for windows
    /// presented through `wl_shm` (e.g. softbuffer): Vulkan and EGL WSI tag the surface
    /// themselves, and a second tag is a protocol error that ends the connection.
    pub fn tag_window_srgb(&mut self, id: WindowId) {
        if let Some(window) = self.windows.get_mut(&id) {
            window.request_srgb();
            self.tag_windows_srgb();
        }
    }

    fn tag_windows_srgb(&mut self) {
        let Some(color) = &self.standard.color else {
            return;
        };
        for window in self.windows.values_mut() {
            window.tag_color(|surface| color.tag_srgb(surface, &self.queue_handle));
        }
    }
}

pub(crate) struct ColorManagerData;

impl Dispatch2<WpColorManagerV1, WaylandPlatform> for ColorManagerData {
    fn event(
        &self,
        state: &mut WaylandPlatform,
        _: &WpColorManagerV1,
        event: wp_color_manager_v1::Event,
        _: &Connection,
        qh: &QueueHandle<WaylandPlatform>,
    ) {
        let Some(color) = state.standard.color.as_mut() else {
            return;
        };
        match event {
            wp_color_manager_v1::Event::SupportedIntent { render_intent } => {
                color.support.add_intent(render_intent);
            }
            wp_color_manager_v1::Event::SupportedFeature { feature } => {
                color.support.add_feature(feature);
            }
            wp_color_manager_v1::Event::SupportedTfNamed { tf } => {
                color.support.add_transfer_function(tf);
            }
            wp_color_manager_v1::Event::SupportedPrimariesNamed { primaries } => {
                color.support.add_primaries(primaries);
            }
            wp_color_manager_v1::Event::Done => {
                if matches!(color.srgb, SrgbDescription::AwaitingSupport) {
                    color.request_srgb_description(qh);
                }
            }
            _ => {}
        }
    }
}

pub(crate) struct ImageDescriptionData;

impl Dispatch2<WpImageDescriptionV1, WaylandPlatform> for ImageDescriptionData {
    fn event(
        &self,
        state: &mut WaylandPlatform,
        description: &WpImageDescriptionV1,
        event: wp_image_description_v1::Event,
        _: &Connection,
        _: &QueueHandle<WaylandPlatform>,
    ) {
        let Some(color) = state.standard.color.as_mut() else {
            return;
        };
        match event {
            wp_image_description_v1::Event::Ready { .. }
            | wp_image_description_v1::Event::Ready2 { .. } => {
                color.srgb = SrgbDescription::Ready(description.clone());
                state.tag_windows_srgb();
            }
            wp_image_description_v1::Event::Failed { .. } => {
                description.destroy();
                color.srgb = SrgbDescription::Unavailable;
            }
            _ => {}
        }
    }
}
