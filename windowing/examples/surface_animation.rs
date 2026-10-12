use std::{
    num::NonZeroU32,
    rc::Rc,
    time::{Duration, Instant},
};

use motion::Spring;
use softbuffer::{Context, SoftBufferError, Surface};
use windowing::{
    ApplicationHandler, ControlFlow, Platform, WindowAttributes, WindowEvent, WindowId,
    dpi::LogicalSize,
    wayland::{
        CrownEvent, EventLoop, SurfaceHandle, WaylandEvent, WaylandPlatform,
        protocols::surface_animation::{
            AnimatedProperty, AnimatedSubsurface, SurfaceAnimationEvent, monotonic_now,
        },
    },
};

const WINDOW_COLOUR: u32 = 0x0020_2428;
const CARD_COLOUR: u32 = 0x0040_90e0;
const CARD_SIZE: u32 = 160;
const CARD_POSITION: i32 = 40;
const ANIMATE_AFTER: Duration = Duration::from_millis(1500);
const RETARGET_AFTER: Duration = Duration::from_millis(200);
const GIVE_UP_AFTER: Duration = Duration::from_secs(12);
const EXPECTED_SETTLES: usize = 3;
const LINGER_AFTER_SETTLING: Duration = Duration::from_millis(1500);

enum Stage {
    Resting { since: Instant },
    Animating { since: Instant },
    Retargeted,
}

struct Card {
    subsurface: Rc<AnimatedSubsurface>,
    _canvas: Surface<SurfaceHandle, Rc<AnimatedSubsurface>>,
    stage: Stage,
}

struct App {
    started: Instant,
    window: Option<WindowId>,
    settled_at: Option<Instant>,
    canvas: Option<Surface<SurfaceHandle, SurfaceHandle>>,
    card: Option<Card>,
    settles: usize,
}

fn softbuffer<T>(result: Result<T, SoftBufferError>) -> anyhow::Result<T> {
    result.map_err(|error| anyhow::anyhow!("softbuffer: {error}"))
}

impl App {
    fn start(&mut self, platform: &mut WaylandPlatform) -> anyhow::Result<()> {
        println!("capabilities: {:?}", platform.capabilities());
        let attributes = WindowAttributes::default()
            .with_title("surface animation")
            .with_app_id("org.crownos.crownui.surface-animation")
            .with_inner_size(LogicalSize::new(480, 320));
        let window = Platform::create_window(platform, attributes)?;
        let handle = platform
            .window(window)
            .map(|window| window.surface_handle())
            .ok_or_else(|| anyhow::anyhow!("window vanished right after creation"))?;
        let context = softbuffer(Context::new(handle.clone()))?;
        self.canvas = Some(softbuffer(Surface::new(&context, handle))?);
        self.window = Some(window);
        Ok(())
    }

    fn create_card(&mut self, platform: &WaylandPlatform, window: WindowId) -> anyhow::Result<()> {
        let window = platform
            .window(window)
            .ok_or_else(|| anyhow::anyhow!("window is gone"))?;
        let subsurface = Rc::new(platform.create_animated_subsurface(window)?);
        subsurface.set_position(CARD_POSITION, CARD_POSITION);
        subsurface.set_desync();

        let context = softbuffer(Context::new(window.surface_handle()))?;
        let mut surface = softbuffer(Surface::new(&context, Rc::clone(&subsurface)))?;
        let side = NonZeroU32::new(CARD_SIZE).ok_or_else(|| anyhow::anyhow!("empty card"))?;
        softbuffer(surface.resize(side, side))?;

        let mut buffer = softbuffer(surface.buffer_mut())?;
        buffer.fill(CARD_COLOUR);
        softbuffer(buffer.present())?;

        self.card = Some(Card {
            subsurface,
            _canvas: surface,
            stage: Stage::Resting {
                since: Instant::now(),
            },
        });
        Ok(())
    }

    fn redraw(&mut self, platform: &mut WaylandPlatform, window: WindowId) -> anyhow::Result<()> {
        if self.card.is_none() {
            self.create_card(platform, window)?;
        }
        let Some(canvas) = self.canvas.as_mut() else {
            return Ok(());
        };
        let mut buffer = softbuffer(canvas.buffer_mut())?;
        buffer.fill(WINDOW_COLOUR);
        softbuffer(buffer.present())
    }

    fn resize(&mut self, width: u32, height: u32) -> anyhow::Result<()> {
        let (Some(canvas), Some(width), Some(height)) = (
            self.canvas.as_mut(),
            NonZeroU32::new(width),
            NonZeroU32::new(height),
        ) else {
            return Ok(());
        };
        softbuffer(canvas.resize(width, height))
    }

    fn step_script(&mut self) {
        let Some(card) = self.card.as_mut() else {
            return;
        };
        let animation = card.subsurface.animation();
        match card.stage {
            Stage::Resting { since } if since.elapsed() >= ANIMATE_AFTER => {
                println!("animating translate_x to 240, scale to 0.6, opacity to 0.5");
                let start = monotonic_now();
                animation.animate(AnimatedProperty::TranslateX, 240.0, Spring::bouncy(), start);
                animation.animate(AnimatedProperty::Scale, 0.6, Spring::smooth(), start);
                animation.animate(AnimatedProperty::Opacity, 0.5, Spring::smooth(), start);
                card.stage = Stage::Animating {
                    since: Instant::now(),
                };
            }
            Stage::Animating { since } if since.elapsed() >= RETARGET_AFTER => {
                println!("retargeting translate_x to 120 mid-flight");
                animation.animate(
                    AnimatedProperty::TranslateX,
                    120.0,
                    Spring::snappy(),
                    monotonic_now(),
                );
                card.stage = Stage::Retargeted;
            }
            _ => return,
        }
        card.subsurface.wl_surface().commit();
    }
}

impl ApplicationHandler<WaylandPlatform> for App {
    fn resumed(&mut self, platform: &mut WaylandPlatform) {
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
            WindowEvent::Resized(size) => self.resize(size.width, size.height),
            WindowEvent::RedrawRequested => self.redraw(platform, window),
            _ => Ok(()),
        };
        if let Err(error) = result {
            eprintln!("{error:#}");
            platform.exit();
        }
    }

    fn platform_event(&mut self, platform: &mut WaylandPlatform, event: WaylandEvent) {
        let WaylandEvent::Crown(CrownEvent::SurfaceAnimation { event, .. }) = event else {
            return;
        };
        match event {
            SurfaceAnimationEvent::Settled { property, value } => {
                println!("settled {property:?} at {value}");
                self.settles += 1;
                if self.settles == EXPECTED_SETTLES {
                    self.settled_at = Some(Instant::now());
                    if let Some(window) = self.window.and_then(|id| platform.window_mut(id)) {
                        window.request_redraw();
                    }
                }
            }
            SurfaceAnimationEvent::Interrupted {
                property,
                value,
                velocity,
            } => println!("interrupted {property:?} at {value} moving {velocity}/s"),
        }
    }

    fn about_to_wait(&mut self, platform: &mut WaylandPlatform) {
        self.step_script();
        if self
            .settled_at
            .is_some_and(|settled| settled.elapsed() >= LINGER_AFTER_SETTLING)
        {
            platform.exit();
        } else if self.started.elapsed() >= GIVE_UP_AFTER {
            eprintln!("gave up waiting for {EXPECTED_SETTLES} settled events");
            platform.exit();
        }
        platform.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_millis(20),
        ));
    }
}

fn main() -> anyhow::Result<()> {
    let mut app = App {
        started: Instant::now(),
        window: None,
        settled_at: None,
        canvas: None,
        card: None,
        settles: 0,
    };
    EventLoop::new()?.run_app(&mut app)?;
    Ok(())
}
