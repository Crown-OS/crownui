use std::ffi::c_void;

use khronos_egl as egl;
use wayland_egl::WlEglSurface;

use super::MSAA_SAMPLES;
use crate::error::InitError;

const EGL_PLATFORM_WAYLAND_KHR: egl::Enum = 0x31D8;
const VSYNC_SWAP_INTERVAL: egl::Int = 1;
const MULTISAMPLE_ATTRIBUTES_LEN: usize = 4;

/// An EGL GLES3 context and window surface, current on the creating thread.
pub(crate) struct EglWindow {
    display: egl::Display,
    surface: egl::Surface,
    context: egl::Context,
}

impl EglWindow {
    /// # Safety
    /// `wl_display` must be a live `wl_display` that outlives the returned window, and
    /// `native_window` must outlive it too.
    pub(crate) unsafe fn new(
        wl_display: *mut c_void,
        native_window: &WlEglSurface,
    ) -> Result<Self, InitError> {
        let api = &egl::API;
        // SAFETY: the caller guarantees `wl_display` is live.
        let display = unsafe {
            api.get_platform_display(EGL_PLATFORM_WAYLAND_KHR, wl_display, &[egl::ATTRIB_NONE])
        }?;
        api.initialize(display)?;
        api.bind_api(egl::OPENGL_ES_API)?;
        let config_attributes = [
            egl::SAMPLE_BUFFERS,
            1,
            egl::SAMPLES,
            MSAA_SAMPLES,
            egl::RED_SIZE,
            8,
            egl::GREEN_SIZE,
            8,
            egl::BLUE_SIZE,
            8,
            egl::ALPHA_SIZE,
            8,
            egl::STENCIL_SIZE,
            8,
            egl::SURFACE_TYPE,
            egl::WINDOW_BIT,
            egl::RENDERABLE_TYPE,
            egl::OPENGL_ES3_BIT,
            egl::NONE,
        ];
        let config = [
            &config_attributes[..],
            &config_attributes[MULTISAMPLE_ATTRIBUTES_LEN..],
        ]
        .into_iter()
        .find_map(|attributes| api.choose_first_config(display, attributes).transpose())
        .transpose()?
        .ok_or(InitError::NoEglConfig)?;
        let context = api.create_context(
            display,
            config,
            None,
            &[egl::CONTEXT_MAJOR_VERSION, 3, egl::NONE],
        )?;
        // SAFETY: the native window is a live wl_egl_window created for this display's surface.
        let surface = match unsafe {
            api.create_platform_window_surface(
                display,
                config,
                native_window.ptr().cast_mut(),
                &[egl::ATTRIB_NONE],
            )
        } {
            Ok(surface) => surface,
            Err(error) => {
                let _ = api.destroy_context(display, context);
                return Err(error.into());
            }
        };
        let window = Self {
            display,
            surface,
            context,
        };
        window.make_current()?;
        api.swap_interval(display, VSYNC_SWAP_INTERVAL)?;
        Ok(window)
    }

    pub(crate) fn make_current(&self) -> Result<(), egl::Error> {
        egl::API.make_current(
            self.display,
            Some(self.surface),
            Some(self.surface),
            Some(self.context),
        )
    }

    pub(crate) fn swap_buffers(&self) -> Result<(), egl::Error> {
        egl::API.swap_buffers(self.display, self.surface)
    }
}

impl Drop for EglWindow {
    fn drop(&mut self) {
        let api = &egl::API;
        let _ = api.make_current(self.display, None, None, None);
        let _ = api.destroy_surface(self.display, self.surface);
        let _ = api.destroy_context(self.display, self.context);
    }
}

pub(crate) fn gl_proc_address(name: &str) -> *mut c_void {
    egl::API
        .get_proc_address(name)
        .map_or(std::ptr::null_mut(), |function| function as *mut c_void)
}
