use std::{
    ffi::{CStr, c_char, c_void},
    path::Path,
};

use khronos_egl as egl;

use crate::error::InitError;

const EGL_PLATFORM_SURFACELESS_MESA: egl::Enum = 0x31DD;
const SURFACELESS_EXTENSION: &str = "EGL_MESA_platform_surfaceless";
const ANY_SURFACE_TYPE: egl::Int = 0;
const EGL_PLATFORM_DEVICE_EXT: egl::Enum = 0x313F;
const EGL_DRM_RENDER_NODE_FILE_EXT: egl::Int = 0x3377;
const DEVICE_EXTENSIONS: [&str; 3] = [
    "EGL_EXT_device_enumeration",
    "EGL_EXT_device_query",
    "EGL_EXT_platform_device",
];
const MAX_DEVICES: usize = 16;

type QueryDevices =
    unsafe extern "system" fn(egl::Int, *mut *mut c_void, *mut egl::Int) -> egl::Boolean;
type QueryDeviceString = unsafe extern "system" fn(*mut c_void, egl::Int) -> *const c_char;

/// How a headless EGL context got a display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeadlessPlatform {
    Surfaceless,
    Pbuffer,
}

/// A GLES3 context current on the creating thread, with no window: on the surfaceless platform
/// it has no surface at all, otherwise a 1x1 pbuffer keeps it current.
pub(crate) struct HeadlessEgl {
    display: egl::Display,
    context: egl::Context,
    pbuffer: Option<egl::Surface>,
    platform: HeadlessPlatform,
}

impl HeadlessEgl {
    pub(crate) fn new() -> Result<Self, InitError> {
        Self::surfaceless().or_else(|surfaceless| {
            tracing::warn!(%surfaceless, "EGL surfaceless platform unavailable, using a pbuffer");
            Self::pbuffer()
        })
    }

    /// A context on the GPU behind `render_node`, so dma-bufs allocated there import without a
    /// copy. Needed on hybrid laptops, where the surfaceless platform may pick the other GPU.
    pub(crate) fn on_render_node(render_node: &Path) -> Result<Self, InitError> {
        let api = &egl::API;
        let client_extensions = api
            .query_string(None, egl::EXTENSIONS)?
            .to_string_lossy()
            .into_owned();
        if let Some(missing) = DEVICE_EXTENSIONS
            .into_iter()
            .find(|name| !client_extensions.split(' ').any(|present| present == *name))
        {
            return Err(InitError::MissingEglExtension(missing));
        }
        let device = find_device(render_node)?;
        // SAFETY: `device` came from eglQueryDevicesEXT and stays valid for the process.
        let display = unsafe {
            api.get_platform_display(EGL_PLATFORM_DEVICE_EXT, device, &[egl::ATTRIB_NONE])
        }?;
        let (headless, _) =
            Self::with_context(display, ANY_SURFACE_TYPE, HeadlessPlatform::Surfaceless)?;
        headless.make_current()?;
        Ok(headless)
    }

    pub(crate) fn display(&self) -> egl::Display {
        self.display
    }

    fn surfaceless() -> Result<Self, InitError> {
        let api = &egl::API;
        let client_extensions = api.query_string(None, egl::EXTENSIONS)?;
        if !client_extensions
            .to_string_lossy()
            .split(' ')
            .any(|name| name == SURFACELESS_EXTENSION)
        {
            return Err(InitError::MissingEglExtension(SURFACELESS_EXTENSION));
        }
        // SAFETY: the surfaceless platform takes no native display.
        let display = unsafe {
            api.get_platform_display(
                EGL_PLATFORM_SURFACELESS_MESA,
                egl::DEFAULT_DISPLAY,
                &[egl::ATTRIB_NONE],
            )
        }?;
        let (headless, _) =
            Self::with_context(display, ANY_SURFACE_TYPE, HeadlessPlatform::Surfaceless)?;
        headless.make_current()?;
        Ok(headless)
    }

    fn pbuffer() -> Result<Self, InitError> {
        let api = &egl::API;
        // SAFETY: EGL_DEFAULT_DISPLAY is always a valid argument.
        let display =
            unsafe { api.get_display(egl::DEFAULT_DISPLAY) }.ok_or(InitError::NoEglDisplay)?;
        let (mut headless, config) =
            Self::with_context(display, egl::PBUFFER_BIT, HeadlessPlatform::Pbuffer)?;
        let size = [egl::WIDTH, 1, egl::HEIGHT, 1, egl::NONE];
        headless.pbuffer = Some(api.create_pbuffer_surface(display, config, &size)?);
        headless.make_current()?;
        Ok(headless)
    }

    fn with_context(
        display: egl::Display,
        surface_type: egl::Int,
        platform: HeadlessPlatform,
    ) -> Result<(Self, egl::Config), InitError> {
        let api = &egl::API;
        api.initialize(display)?;
        api.bind_api(egl::OPENGL_ES_API)?;
        let config = choose_config(display, surface_type)?;
        let context = api.create_context(
            display,
            config,
            None,
            &[egl::CONTEXT_MAJOR_VERSION, 3, egl::NONE],
        )?;
        let headless = Self {
            display,
            context,
            pbuffer: None,
            platform,
        };
        Ok((headless, config))
    }

    pub(crate) fn platform(&self) -> HeadlessPlatform {
        self.platform
    }

    pub(crate) fn make_current(&self) -> Result<(), egl::Error> {
        egl::API.make_current(self.display, self.pbuffer, self.pbuffer, Some(self.context))
    }
}

/// The EGL device whose DRM render node is `render_node`.
fn find_device(render_node: &Path) -> Result<*mut c_void, InitError> {
    let api = &egl::API;
    let missing = || InitError::NoRenderNodeDevice(render_node.display().to_string());
    let query_devices = api
        .get_proc_address("eglQueryDevicesEXT")
        .ok_or_else(missing)?;
    let query_string = api
        .get_proc_address("eglQueryDeviceStringEXT")
        .ok_or_else(missing)?;
    // SAFETY: both names resolve to these signatures per EGL_EXT_device_enumeration/query.
    let (query_devices, query_string): (QueryDevices, QueryDeviceString) = unsafe {
        (
            std::mem::transmute::<extern "system" fn(), QueryDevices>(query_devices),
            std::mem::transmute::<extern "system" fn(), QueryDeviceString>(query_string),
        )
    };
    let mut devices = [std::ptr::null_mut(); MAX_DEVICES];
    let mut count: egl::Int = 0;
    // SAFETY: `devices` holds MAX_DEVICES entries and `count` receives how many were written.
    if unsafe { query_devices(MAX_DEVICES as egl::Int, devices.as_mut_ptr(), &mut count) }
        != egl::TRUE
    {
        return Err(missing());
    }
    let wanted = render_node
        .canonicalize()
        .unwrap_or_else(|_| render_node.to_path_buf());
    devices
        .into_iter()
        .take(usize::try_from(count).unwrap_or(0))
        .find(|device| {
            // SAFETY: a device from the query above; the string is static per EGL.
            let name = unsafe { query_string(*device, EGL_DRM_RENDER_NODE_FILE_EXT) };
            !name.is_null()
                && Path::new(&*unsafe { CStr::from_ptr(name) }.to_string_lossy())
                    .canonicalize()
                    .ok()
                    .as_deref()
                    == Some(wanted.as_path())
        })
        .ok_or_else(missing)
}

fn choose_config(display: egl::Display, surface_type: egl::Int) -> Result<egl::Config, InitError> {
    let attributes = [
        egl::RED_SIZE,
        8,
        egl::GREEN_SIZE,
        8,
        egl::BLUE_SIZE,
        8,
        egl::ALPHA_SIZE,
        8,
        egl::SURFACE_TYPE,
        surface_type,
        egl::RENDERABLE_TYPE,
        egl::OPENGL_ES3_BIT,
        egl::NONE,
    ];
    egl::API
        .choose_first_config(display, &attributes)?
        .ok_or(InitError::NoEglConfig)
}

impl Drop for HeadlessEgl {
    fn drop(&mut self) {
        let api = &egl::API;
        let _ = api.make_current(self.display, None, None, None);
        if let Some(pbuffer) = self.pbuffer {
            let _ = api.destroy_surface(self.display, pbuffer);
        }
        let _ = api.destroy_context(self.display, self.context);
    }
}
