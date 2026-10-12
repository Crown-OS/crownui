use std::num::NonZeroU32;

use input::{Key, NamedKey};
use softbuffer::{Context, SoftBufferError, Surface};
use windowing::{
    ApplicationHandler, Material, Platform, WindowAttributes, WindowControl, WindowEvent,
    WindowId,
    dpi::LogicalSize,
    wayland::{EventLoop, SurfaceHandle, WaylandEvent, WaylandPlatform},
};

const BACKGROUND: u32 = 0x0020_2430;

struct Canvas {
    window: WindowId,
    surface: Surface<SurfaceHandle, SurfaceHandle>,
}

#[derive(Default)]
struct App {
    canvas: Option<Canvas>,
}

fn softbuffer<T>(result: Result<T, SoftBufferError>) -> anyhow::Result<T> {
    result.map_err(|error| anyhow::anyhow!("softbuffer: {error}"))
}

impl App {
    fn create_canvas(platform: &mut WaylandPlatform) -> anyhow::Result<Canvas> {
        let attributes = WindowAttributes::default()
            .with_title("CrownUI")
            .with_app_id("org.crownos.crownui.example")
            .with_inner_size(LogicalSize::new(640, 400))
            .with_material(Material::thin());
        let window = Platform::create_window(platform, attributes)?;
        platform.tag_window_srgb(window);
        let handle = platform
            .window(window)
            .map(|window| window.surface_handle())
            .ok_or_else(|| anyhow::anyhow!("window vanished right after creation"))?;
        let context = softbuffer(Context::new(handle.clone()))?;
        let surface = softbuffer(Surface::new(&context, handle))?;
        Ok(Canvas { window, surface })
    }

    fn redraw(&mut self, platform: &mut WaylandPlatform) -> anyhow::Result<()> {
        let Some(canvas) = self.canvas.as_mut() else {
            return Ok(());
        };
        let mut buffer = softbuffer(canvas.surface.buffer_mut())?;
        buffer.fill(BACKGROUND);
        if let Some(window) = platform.window_mut(canvas.window) {
            window.pre_present_notify();
        }
        softbuffer(buffer.present())?;
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) -> anyhow::Result<()> {
        let (Some(canvas), Some(width), Some(height)) = (
            self.canvas.as_mut(),
            NonZeroU32::new(width),
            NonZeroU32::new(height),
        ) else {
            return Ok(());
        };
        softbuffer(canvas.surface.resize(width, height))?;
        Ok(())
    }
}

impl ApplicationHandler<WaylandPlatform> for App {
    fn resumed(&mut self, platform: &mut WaylandPlatform) {
        println!("capabilities: {:?}", platform.capabilities());
        println!("screencast: {:?}", platform.screencast_capabilities());
        match Self::create_canvas(platform) {
            Ok(canvas) => self.canvas = Some(canvas),
            Err(error) => {
                eprintln!("failed to create the window: {error:#}");
                platform.exit();
            }
        }
    }

    fn window_event(&mut self, platform: &mut WaylandPlatform, _: WindowId, event: WindowEvent) {
        let result = match event {
            WindowEvent::CloseRequested => {
                platform.exit();
                Ok(())
            }
            WindowEvent::KeyDown(key) if key.key == Key::Named(NamedKey::Escape) => {
                platform.exit();
                Ok(())
            }
            WindowEvent::KeyDown(key) if key.modifiers.ctrl_key && key.key == Key::Character('c') => {
                platform.set_clipboard_text("Copied from CrownUI".to_owned());
                Ok(())
            }
            WindowEvent::KeyDown(key) if key.modifiers.ctrl_key && key.key == Key::Character('v') => {
                platform.request_clipboard_text();
                Ok(())
            }
            WindowEvent::ClipboardText(text) => {
                println!("clipboard: {text:?}");
                Ok(())
            }
            WindowEvent::TouchpadGesture(gesture) => {
                println!("gesture: {gesture:?}");
                Ok(())
            }
            WindowEvent::PointerDown(_) => {
                if let Some(canvas) = &self.canvas
                    && let Some(window) = platform.window_mut(canvas.window)
                {
                    window.begin_move();
                }
                Ok(())
            }
            WindowEvent::Resized(size) => self.resize(size.width, size.height),
            WindowEvent::RedrawRequested => self.redraw(platform),
            WindowEvent::ScaleFactorChanged(scale) => {
                println!("scale factor {}", scale.get());
                Ok(())
            }
            _ => Ok(()),
        };
        if let Err(error) = result {
            eprintln!("{error:#}");
            platform.exit();
        }
    }

    fn platform_event(&mut self, _: &mut WaylandPlatform, event: WaylandEvent) {
        println!("platform event: {event:?}");
    }
}

fn main() -> anyhow::Result<()> {
    EventLoop::new()?.run_app(&mut App::default())?;
    Ok(())
}
