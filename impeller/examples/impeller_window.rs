use std::time::Instant;

use anyhow::{Context as _, Result};
use geometry::{CornerRadii, Point, Rect, Transform, Vector, rgb, rgba};
use graphics::{
    BoundaryId, Brush, CompositeOp, Composition, CreateRenderer, DrawCommand, DrawList,
    GradientStop, GraphicsBackend, ImageId, LayerProps, Paint, Renderer, TextLayout, TextLayoutId,
    TextStyle, TextSystem,
};
use impeller::{ImpellerBackend, ImpellerRenderer};
use input::{Key, NamedKey};
use windowing::{
    ApplicationHandler, Platform, PlatformSurface, WindowAttributes, WindowEvent, WindowId,
    dpi::LogicalSize,
    wayland::{EventLoop, RenderSurface, WaylandPlatform},
};

const SCENE: BoundaryId = BoundaryId(1);
const BADGE: BoundaryId = BoundaryId(2);
const TITLE: TextLayoutId = TextLayoutId(1);
const CAPTION: TextLayoutId = TextLayoutId(2);
const CHECKER: ImageId = ImageId(1);
const CHECKER_SIZE: u32 = 16;
const LOG_EVERY_FRAMES: u64 = 60;
const FRAME_LIMIT_ENV: &str = "IMPELLER_EXAMPLE_FRAMES";

struct Scene {
    window: WindowId,
    renderer: ImpellerRenderer,
    surface: RenderSurface,
    started: Instant,
    frames: u64,
    awaiting_frame_callback: bool,
    frame_limit: Option<u64>,
}

#[derive(Default)]
struct App {
    scene: Option<Scene>,
}

fn checker_pixels() -> Vec<u8> {
    (0..CHECKER_SIZE * CHECKER_SIZE)
        .flat_map(|index| {
            let light = (index % CHECKER_SIZE / 4 + index / CHECKER_SIZE / 4).is_multiple_of(2);
            if light {
                [250, 204, 21, 255]
            } else {
                [30, 41, 59, 160]
            }
        })
        .collect()
}

fn gradient(start: Point, end: Point, from: u32, to: u32) -> Paint {
    Paint {
        brush: Brush::LinearGradient {
            start,
            end,
            stops: [
                GradientStop {
                    offset: 0.0,
                    color: rgb(from),
                },
                GradientStop {
                    offset: 1.0,
                    color: rgb(to),
                },
            ]
            .into_iter()
            .collect(),
        },
        blur: None,
    }
}

fn scene_list(width: f32, height: f32, seconds: f32) -> DrawList {
    let mut list = DrawList::new();
    let viewport = Rect::from_xywh(0.0, 0.0, width, height);
    list.push(DrawCommand::FillRect(
        viewport,
        gradient(
            Point::new(0.0, 0.0),
            Point::new(width, height),
            0x1e1b4b,
            0x831843,
        ),
    ));
    list.push(DrawCommand::FillRoundedRect(
        Rect::from_xywh(width * 0.6, 40.0, 140.0, 140.0),
        CornerRadii::uniform(70.0),
        Paint {
            blur: Some(18.0),
            ..Paint::solid(rgba(0x22d3eecc))
        },
    ));

    let card_radii = CornerRadii::uniform(24.0);
    let card_x = 32.0 + 80.0 * (seconds.sin() * 0.5 + 0.5);
    for (index, color) in [0xf97316, 0x10b981].into_iter().enumerate() {
        let card = Rect::from_xywh(card_x + index as f32 * 240.0, 48.0, 210.0, 140.0);
        list.shadow(
            card,
            card_radii,
            rgba(0x00000080),
            32.0,
            Vector::new(0.0, 12.0),
        );
        list.fill(card, card_radii, rgb(color));
        list.stroke(card, card_radii, 1.5, rgba(0xffffff66));
    }
    list.push(DrawCommand::Image(
        CHECKER,
        Rect::from_xywh(card_x + 20.0, 120.0, 48.0, 48.0),
    ));
    list.text(TITLE, Point::new(card_x + 20.0, 64.0));

    let glass = Rect::from_xywh(24.0, 150.0, (width - 48.0).max(0.0), 120.0);
    let glass_radii = CornerRadii::uniform(20.0);
    list.save();
    list.clip(glass, glass_radii);
    list.push(DrawCommand::SaveLayer {
        bounds: Some(glass),
        opacity: 1.0,
        backdrop_blur: Some(12.0),
    });
    list.fill(glass, CornerRadii::ZERO, rgba(0xffffff26));
    list.restore();
    list.stroke(glass, glass_radii, 1.0, rgba(0xffffff80));
    list.text(
        CAPTION,
        Point::new(glass.min_x() + 20.0, glass.min_y() + 20.0),
    );
    list.restore();
    list
}

fn badge_list() -> DrawList {
    let mut list = DrawList::new();
    let badge = Rect::from_xywh(-40.0, -14.0, 80.0, 28.0);
    list.fill(badge, CornerRadii::uniform(14.0), rgb(0x6366f1));
    list.shadow(
        badge,
        CornerRadii::uniform(14.0),
        rgba(0x6366f166),
        12.0,
        Vector::ZERO,
    );
    list
}

impl Scene {
    fn create(platform: &mut WaylandPlatform) -> Result<Self> {
        let attributes = WindowAttributes::default()
            .with_title("CrownUI on Impeller")
            .with_app_id("org.crownos.crownui.impeller-example")
            .with_inner_size(LogicalSize::new(720, 420));
        let window = Platform::create_window(platform, attributes)?;
        let surface = platform
            .surface(window)
            .context("window vanished right after creation")?;
        let mut renderer = ImpellerBackend::create_renderer(&surface)?;
        eprintln!("[impeller_window] backend {:?}", renderer.api());

        let mut text = ImpellerBackend::create_text_system()?;
        let title_style = TextStyle {
            font_size: 17.0,
            font_weight: graphics::FontWeight::SEMIBOLD,
            color: rgb(0xffffff),
            ..TextStyle::default()
        };
        let title = text.layout(
            &format!("Hello CrownUI — {:?}", renderer.api()),
            &title_style,
            Some(180.0),
        );
        let caption_style = TextStyle {
            font_size: 15.0,
            color: rgb(0xf8fafc),
            line_height: Some(1.4),
            max_lines: Some(2),
            ellipsis: true,
            ..TextStyle::default()
        };
        let caption = text.layout(
            "Backdrop-blurred glass over gradient cards with CSS box shadows, recorded once per frame \
             into an Impeller display list and composited at the surface scale.",
            &caption_style,
            Some(420.0),
        );
        eprintln!(
            "[impeller_window] title {:?} lines={} | caption {:?} lines={}",
            title.size(),
            title.line_count(),
            caption.size(),
            caption.line_count()
        );
        renderer.upload_text(TITLE, title);
        renderer.upload_text(CAPTION, caption);
        renderer.upload_image(CHECKER, CHECKER_SIZE, CHECKER_SIZE, &checker_pixels())?;
        renderer.record_boundary(BADGE, &badge_list());

        let (size, scale) = platform
            .window(window)
            .map(|window| (window.inner_size(), window.scale_factor().get() as f32))
            .context("window vanished right after creation")?;
        renderer.set_viewport(size.width, size.height, scale);
        Ok(Self {
            window,
            renderer,
            surface,
            started: Instant::now(),
            frames: 0,
            awaiting_frame_callback: false,
            frame_limit: std::env::var(FRAME_LIMIT_ENV)
                .ok()
                .and_then(|limit| limit.parse().ok()),
        })
    }

    fn composition(&self, seconds: f32) -> Composition {
        let (width, _, scale) = self.renderer.viewport();
        let badge_layer = LayerProps {
            transform: Transform::rotate(seconds * 0.8).then(Transform::translate(Vector::new(
                width as f32 / scale - 70.0,
                310.0,
            ))),
            opacity: 0.6 + 0.4 * seconds.cos().abs(),
            ..LayerProps::default()
        };
        Composition {
            clear_color: rgba(0x00000000),
            ops: vec![
                CompositeOp::PushLayer(LayerProps::default()),
                CompositeOp::Draw(SCENE),
                CompositeOp::PopLayer,
                CompositeOp::PushLayer(badge_layer),
                CompositeOp::Draw(BADGE),
                CompositeOp::PopLayer,
            ],
        }
    }

    fn redraw(&mut self) -> Result<()> {
        if self.awaiting_frame_callback {
            self.surface.wait_for_frame()?;
        }
        let (width, height, scale) = self.renderer.viewport();
        let seconds = self.started.elapsed().as_secs_f32();
        self.renderer.record_boundary(
            SCENE,
            &scene_list(width as f32 / scale, height as f32 / scale, seconds),
        );
        let composition = self.composition(seconds);
        let presented_before = self.renderer.frames_presented();
        self.surface.pre_present_notify();
        self.renderer.render(&composition)?;
        self.awaiting_frame_callback = self.renderer.frames_presented() > presented_before;
        self.frames += 1;
        if self.frames == 1 || self.frames.is_multiple_of(LOG_EVERY_FRAMES) {
            eprintln!(
                "[impeller_window] frame {} api={:?} viewport={width}x{height}@{scale} presented_size={:?} \
                 presented={} fps={:.1}",
                self.frames,
                self.renderer.api(),
                self.renderer.presented_size(),
                self.renderer.frames_presented(),
                self.frames as f32 / seconds.max(f32::EPSILON),
            );
        }
        Ok(())
    }

    fn done(&self) -> bool {
        self.frame_limit.is_some_and(|limit| self.frames >= limit)
    }
}

impl App {
    fn handle(&mut self, platform: &mut WaylandPlatform, event: WindowEvent) -> Result<()> {
        let Some(scene) = self.scene.as_mut() else {
            return Ok(());
        };
        let exit_requested = match event {
            WindowEvent::CloseRequested => true,
            WindowEvent::KeyDown(key) => key.key == Key::Named(NamedKey::Escape),
            WindowEvent::Resized(size) => {
                let scale = platform
                    .window(scene.window)
                    .map_or(1.0, |window| window.scale_factor().get() as f32);
                eprintln!(
                    "[impeller_window] resized to {}x{} @{scale}",
                    size.width, size.height
                );
                scene.renderer.set_viewport(size.width, size.height, scale);
                false
            }
            WindowEvent::RedrawRequested => {
                scene.redraw()?;
                if let Some(window) = platform.window_mut(scene.window) {
                    window.request_redraw();
                }
                scene.done()
            }
            _ => false,
        };
        if exit_requested {
            self.shut_down(platform);
        }
        Ok(())
    }

    fn shut_down(&mut self, platform: &mut WaylandPlatform) {
        if let Some(scene) = self.scene.take() {
            eprintln!(
                "[impeller_window] exiting after {} frames ({} presented) on {:?}",
                scene.frames,
                scene.renderer.frames_presented(),
                scene.renderer.api()
            );
        }
        platform.exit();
    }
}

impl ApplicationHandler<WaylandPlatform> for App {
    fn resumed(&mut self, platform: &mut WaylandPlatform) {
        match Scene::create(platform) {
            Ok(scene) => self.scene = Some(scene),
            Err(error) => {
                eprintln!("[impeller_window] failed to start: {error:#}");
                platform.exit();
            }
        }
    }

    fn window_event(&mut self, platform: &mut WaylandPlatform, _: WindowId, event: WindowEvent) {
        if let Err(error) = self.handle(platform, event) {
            eprintln!("[impeller_window] {error:#}");
            self.shut_down(platform);
        }
    }
}

fn main() -> Result<()> {
    EventLoop::new()?.run_app(&mut App::default())?;
    Ok(())
}
