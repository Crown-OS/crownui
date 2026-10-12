use std::num::NonZeroU32;

use sctk::{
    compositor::CompositorHandler,
    reexports::client::{
        Connection, QueueHandle,
        protocol::{wl_output::Transform, wl_output::WlOutput, wl_surface::WlSurface},
    },
    shell::{
        WaylandSurface,
        wlr_layer::{LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
        xdg::{
            XdgSurface,
            window::{Window as XdgWindow, WindowConfigure, WindowHandler},
        },
    },
};

use super::{Window, shell, surface_window_id};
use crate::{
    WindowEvent, WindowId,
    dpi::{LogicalSize, ScaleFactor},
    wayland::event_loop::{QueuedEvent, WaylandPlatform},
};

impl Window {
    fn configure(&mut self, configure: &WindowConfigure, events: &mut Vec<QueuedEvent>) {
        let (width, height) = configure.new_size;
        let size = LogicalSize::new(
            width.map_or(self.floating_size.width, NonZeroU32::get),
            height.map_or(self.floating_size.height, NonZeroU32::get),
        );
        if !(configure.is_maximized() || configure.is_fullscreen() || configure.is_tiled()) {
            self.floating_size = size;
        }
        if let Some(toplevel) = self.shell.toplevel() {
            toplevel
                .xdg_surface()
                .set_window_geometry(0, 0, size.width as i32, size.height as i32);
        }
        self.resize_to(size, events);
    }

    fn configure_layer(&mut self, (width, height): (u32, u32), events: &mut Vec<QueuedEvent>) {
        let size = LogicalSize::new(
            if width == 0 {
                self.logical_size.width
            } else {
                width
            },
            if height == 0 {
                self.logical_size.height
            } else {
                height
            },
        );
        self.resize_to(size, events);
    }

    fn resize_to(&mut self, size: LogicalSize, events: &mut Vec<QueuedEvent>) {
        let resized = !self.configured || self.logical_size != size;
        self.logical_size = size;
        self.scaling
            .apply(self.shell.wl_surface(), size, self.scale_factor);
        self.configured = true;

        if resized {
            events.push(QueuedEvent::Window(
                self.id,
                WindowEvent::Resized(self.inner_size()),
            ));
        }
        self.request_redraw();
    }

    fn rescale(&mut self, scale: ScaleFactor, events: &mut Vec<QueuedEvent>) {
        if self.scale_factor == scale {
            return;
        }
        self.scale_factor = scale;
        self.scaling
            .apply(self.shell.wl_surface(), self.logical_size, scale);
        if !self.configured {
            return;
        }
        events.extend([
            QueuedEvent::Window(self.id, WindowEvent::ScaleFactorChanged(scale)),
            QueuedEvent::Window(self.id, WindowEvent::Resized(self.inner_size())),
        ]);
        self.request_redraw();
    }
}

impl WaylandPlatform {
    pub(crate) fn queue_requested_redraws(&mut self) {
        let ready = self.windows.values_mut().filter_map(|window| {
            let ready = window.configured && !window.frame_callback_pending;
            (ready && std::mem::take(&mut window.redraw_requested)).then_some(window.id)
        });
        self.pending_events
            .extend(ready.map(|id| QueuedEvent::Window(id, WindowEvent::RedrawRequested)));
    }

    pub(crate) fn apply_window_scale(&mut self, window_id: WindowId, scale: ScaleFactor) {
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.rescale(scale, &mut self.pending_events);
        }
    }

    /// Asks every layer surface sized by its output for the output's current size.
    pub(crate) fn refit_layers(&mut self) {
        let output = shell::primary_output_size(self.outputs());
        for window in self.windows.values_mut() {
            let (Some(options), super::ShellSurface::Layer(layer)) = (&window.layer, &window.shell)
            else {
                continue;
            };
            if !options.follows_output() {
                continue;
            }
            let fallback = (window.logical_size.width, window.logical_size.height);
            let (width, height) = options.requested_size(output, fallback);
            layer.set_size(width, height);
            layer.commit();
        }
    }

    pub(crate) fn push_window_event(&mut self, window_id: WindowId, event: WindowEvent) {
        self.pending_events
            .push(QueuedEvent::Window(window_id, event));
    }
}

impl CompositorHandler for WaylandPlatform {
    fn scale_factor_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        surface: &WlSurface,
        new_factor: i32,
    ) {
        let window_id = surface_window_id(surface);
        let uses_integer_scale = self
            .windows
            .get(&window_id)
            .is_some_and(|window| !window.uses_fractional_scale());
        if uses_integer_scale {
            let factor = u32::try_from(new_factor).unwrap_or(1).max(1);
            self.apply_window_scale(window_id, ScaleFactor::from_integer(factor));
        }
    }

    fn transform_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlSurface,
        _: Transform,
    ) {
    }

    fn frame(&mut self, _: &Connection, _: &QueueHandle<Self>, surface: &WlSurface, _: u32) {
        if let Some(window) = self.windows.get_mut(&surface_window_id(surface)) {
            window.frame_callback_pending = false;
        }
    }

    fn surface_enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlSurface,
        _: &WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlSurface,
        _: &WlOutput,
    ) {
    }
}

impl WindowHandler for WaylandPlatform {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, window: &XdgWindow) {
        self.push_window_event(
            surface_window_id(window.wl_surface()),
            WindowEvent::CloseRequested,
        );
    }

    fn configure(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        window: &XdgWindow,
        configure: WindowConfigure,
        _: u32,
    ) {
        if let Some(state) = self
            .windows
            .get_mut(&surface_window_id(window.wl_surface()))
        {
            state.configure(&configure, &mut self.pending_events);
        }
    }
}

impl LayerShellHandler for WaylandPlatform {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, layer: &LayerSurface) {
        self.push_window_event(
            surface_window_id(layer.wl_surface()),
            WindowEvent::CloseRequested,
        );
    }

    fn configure(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _: u32,
    ) {
        if let Some(state) = self.windows.get_mut(&surface_window_id(layer.wl_surface())) {
            state.configure_layer(configure.new_size, &mut self.pending_events);
        }
    }
}
