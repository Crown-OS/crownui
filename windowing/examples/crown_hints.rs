use std::{
    collections::HashMap,
    num::NonZeroU32,
    time::{Duration, Instant},
};

use softbuffer::{Context, SoftBufferError, Surface};
use windowing::{
    ApplicationHandler, ControlFlow, Platform, WindowAttributes, WindowEvent, WindowId,
    dpi::LogicalSize,
    wayland::{
        CrownEvent, EventLoop, SurfaceHandle, WaylandEvent, WaylandPlatform,
        protocols::{
            agent_access::AgentAccessRequest, surface_visibility::SurfaceVisibilityObserver,
        },
    },
};

const OBSERVED_COLOUR: u32 = 0x0030_6090;
const COVER_COLOUR: u32 = 0x0090_3030;
const COVER_AFTER: Duration = Duration::from_secs(2);
const UNCOVER_AFTER: Duration = Duration::from_secs(5);
const EXIT_AFTER: Duration = Duration::from_secs(13);

struct Canvas {
    surface: Surface<SurfaceHandle, SurfaceHandle>,
    colour: u32,
}

struct App {
    started: Instant,
    canvases: HashMap<WindowId, Canvas>,
    observed: Option<WindowId>,
    cover: Option<WindowId>,
    covered_once: bool,
    observer: Option<SurfaceVisibilityObserver>,
    access: Option<AgentAccessRequest>,
}

fn softbuffer<T>(result: Result<T, SoftBufferError>) -> anyhow::Result<T> {
    result.map_err(|error| anyhow::anyhow!("softbuffer: {error}"))
}

impl App {
    fn open(
        &mut self,
        platform: &mut WaylandPlatform,
        title: &str,
        colour: u32,
    ) -> anyhow::Result<WindowId> {
        let attributes = WindowAttributes::default()
            .with_title(title)
            .with_app_id("org.crownos.crownui.hints")
            .with_inner_size(LogicalSize::new(480, 320));
        let window = Platform::create_window(platform, attributes)?;
        let handle = platform
            .window(window)
            .map(|window| window.surface_handle())
            .ok_or_else(|| anyhow::anyhow!("window vanished right after creation"))?;
        let context = softbuffer(Context::new(handle.clone()))?;
        let surface = softbuffer(Surface::new(&context, handle))?;
        self.canvases.insert(window, Canvas { surface, colour });
        Ok(window)
    }

    fn start(&mut self, platform: &mut WaylandPlatform) -> anyhow::Result<()> {
        let observed = self.open(platform, "observed", OBSERVED_COLOUR)?;
        self.observed = Some(observed);
        if let Some(window) = platform.window(observed) {
            self.observer = Some(platform.observe_visibility(window)?);
        }
        self.access = Some(platform.create_agent_access_request(
            "org.crownos.crownui.hints",
            "print the agent access decision",
        )?);
        Ok(())
    }

    fn redraw(&mut self, platform: &mut WaylandPlatform, window: WindowId) -> anyhow::Result<()> {
        let Some(canvas) = self.canvases.get_mut(&window) else {
            return Ok(());
        };
        let mut buffer = softbuffer(canvas.surface.buffer_mut())?;
        buffer.fill(canvas.colour);
        if let Some(window) = platform.window_mut(window) {
            window.pre_present_notify();
        }
        softbuffer(buffer.present())
    }

    fn resize(&mut self, window: WindowId, width: u32, height: u32) -> anyhow::Result<()> {
        let (Some(canvas), Some(width), Some(height)) = (
            self.canvases.get_mut(&window),
            NonZeroU32::new(width),
            NonZeroU32::new(height),
        ) else {
            return Ok(());
        };
        softbuffer(canvas.surface.resize(width, height))
    }

    fn advance_script(&mut self, platform: &mut WaylandPlatform) -> anyhow::Result<()> {
        let elapsed = self.started.elapsed();
        if elapsed >= EXIT_AFTER {
            platform.exit();
        } else if elapsed >= UNCOVER_AFTER {
            if let Some(cover) = self.cover.take() {
                println!("closing the fullscreen cover");
                self.canvases.remove(&cover);
                platform.close_window(cover);
            }
        } else if elapsed >= COVER_AFTER && !self.covered_once {
            self.covered_once = true;
            println!("opening a fullscreen cover");
            let cover = self.open(platform, "cover", COVER_COLOUR)?;
            if let Some(window) = platform.window(cover) {
                window.set_fullscreen(None);
            }
            self.cover = Some(cover);
        }
        platform.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_millis(100),
        ));
        Ok(())
    }
}

impl ApplicationHandler<WaylandPlatform> for App {
    fn resumed(&mut self, platform: &mut WaylandPlatform) {
        println!("capabilities: {:?}", platform.capabilities());
        if let Err(error) = self.start(platform) {
            eprintln!("failed to start: {error:#}");
            platform.exit();
        }
    }

    fn window_event(
        &mut self,
        platform: &mut WaylandPlatform,
        window: WindowId,
        event: WindowEvent,
    ) {
        let result = match event {
            WindowEvent::CloseRequested => {
                platform.exit();
                Ok(())
            }
            WindowEvent::Resized(size) => self.resize(window, size.width, size.height),
            WindowEvent::RedrawRequested => self.redraw(platform, window),
            _ => Ok(()),
        };
        if let Err(error) = result {
            eprintln!("{error:#}");
            platform.exit();
        }
    }

    fn platform_event(&mut self, _: &mut WaylandPlatform, event: WaylandEvent) {
        match event {
            WaylandEvent::Crown(CrownEvent::SurfaceVisibility {
                window,
                state,
                thumbnail_scale,
            }) => println!(
                "visibility: observed={} {state:?} thumbnail_scale={thumbnail_scale}",
                Some(window) == self.observed
            ),
            WaylandEvent::Crown(CrownEvent::AgentAccess { decision, .. }) => {
                println!("agent access: {decision:?}");
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, platform: &mut WaylandPlatform) {
        if let Err(error) = self.advance_script(platform) {
            eprintln!("{error:#}");
            platform.exit();
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut app = App {
        started: Instant::now(),
        canvases: HashMap::new(),
        observed: None,
        cover: None,
        covered_once: false,
        observer: None,
        access: None,
    };
    EventLoop::new()?.run_app(&mut app)?;
    Ok(())
}
