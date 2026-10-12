use sctk::{
    reexports::client::{QueueHandle, protocol::wl_surface::WlSurface},
    shell::{
        WaylandSurface,
        wlr_layer::{self, LayerShell, LayerSurface},
        xdg::window::Window as XdgWindow,
    },
};

use crate::{
    KeyboardInteractivity, Layer, LayerOptions,
    wayland::{event_loop::WaylandPlatform, output::Output},
};

/// The role a window's `wl_surface` plays; cloning keeps the surface alive.
#[derive(Debug, Clone)]
pub(crate) enum ShellSurface {
    Toplevel(XdgWindow),
    Layer(LayerSurface),
}

impl ShellSurface {
    pub(crate) fn wl_surface(&self) -> &WlSurface {
        match self {
            Self::Toplevel(toplevel) => toplevel.wl_surface(),
            Self::Layer(layer) => layer.wl_surface(),
        }
    }

    pub(crate) fn toplevel(&self) -> Option<&XdgWindow> {
        match self {
            Self::Toplevel(toplevel) => Some(toplevel),
            Self::Layer(_) => None,
        }
    }

    pub(crate) fn commit(&self) {
        self.wl_surface().commit();
    }
}

pub(crate) fn create_layer(
    layer_shell: &LayerShell,
    surface: WlSurface,
    options: &LayerOptions,
    size: (u32, u32),
    qh: &QueueHandle<WaylandPlatform>,
) -> LayerSurface {
    let layer = layer_shell.create_layer_surface(
        qh,
        surface,
        sctk_layer(options.layer),
        Some(options.namespace.clone()),
        None,
    );
    layer.set_anchor(wlr_layer::Anchor::from_bits_truncate(options.anchor.bits()));
    layer.set_size(size.0, size.1);
    layer.set_exclusive_zone(options.exclusive_zone);
    let margin = options.margin;
    layer.set_margin(margin.top, margin.right, margin.bottom, margin.left);
    layer.set_keyboard_interactivity(sctk_keyboard(options.keyboard));
    layer
}

/// The logical size of the output a layer surface would land on before the compositor says which.
pub(crate) fn primary_output_size(outputs: impl Iterator<Item = Output>) -> Option<(u32, u32)> {
    outputs
        .filter_map(|output| output.logical_size)
        .map(|(width, height)| (width.max(0) as u32, height.max(0) as u32))
        .find(|&(width, height)| width > 0 && height > 0)
}

fn sctk_layer(layer: Layer) -> wlr_layer::Layer {
    match layer {
        Layer::Background => wlr_layer::Layer::Background,
        Layer::Bottom => wlr_layer::Layer::Bottom,
        Layer::Top => wlr_layer::Layer::Top,
        Layer::Overlay => wlr_layer::Layer::Overlay,
    }
}

fn sctk_keyboard(keyboard: KeyboardInteractivity) -> wlr_layer::KeyboardInteractivity {
    match keyboard {
        KeyboardInteractivity::None => wlr_layer::KeyboardInteractivity::None,
        KeyboardInteractivity::Exclusive => wlr_layer::KeyboardInteractivity::Exclusive,
        KeyboardInteractivity::OnDemand => wlr_layer::KeyboardInteractivity::OnDemand,
    }
}
