use std::time::Duration;

use geometry::{CornerShape, Size};
use graphics::{CreateRenderer, GraphicsBackend, GraphicsError};
use layouting::LayoutEngine;
use semantics::{SemanticId, UiAction};
use view::{ActionError, LayoutState, TextCache, Ui, View};
use windowing::{SurfaceState, WindowEvent};

use crate::{
    frame::build_batch,
    render::{RenderStats, RenderTree},
};

const FRAME_INTERVAL: Duration = Duration::from_micros(16_667);
const SETTLE_LIMIT: Duration = Duration::from_secs(10);

/// A whole app driven synchronously on one thread with a manual clock: the harness behind
/// `crowntest`, previews and golden tests. Works with any backend that can render without a
/// window (`RecordingBackend`).
pub struct InlineApp<G: GraphicsBackend, L: LayoutEngine> {
    ui: Ui,
    layout: LayoutState<L>,
    text: TextCache<G::TextSystem>,
    tree: RenderTree,
    renderer: G::Renderer,
    scale: f32,
    now: Duration,
    last_surface: Option<SurfaceState>,
    last_stats: RenderStats,
}

impl<G: GraphicsBackend + CreateRenderer<()>, L: LayoutEngine> InlineApp<G, L> {
    /// Builds `root` into a `viewport`-sized window and renders the first frame.
    pub fn new(root: impl View, engine: L, viewport: Size) -> Result<Self, GraphicsError> {
        Self::with_surface(root, engine, viewport, 1.0, &())
    }
}

impl<G: GraphicsBackend, L: LayoutEngine> InlineApp<G, L> {
    /// Like [`InlineApp::new`], rendering into `surface` (e.g. an offscreen GPU target) at `scale`.
    pub fn with_surface<S>(
        root: impl View,
        engine: L,
        viewport: Size,
        scale: f32,
        surface: &S,
    ) -> Result<Self, GraphicsError>
    where
        G: CreateRenderer<S>,
    {
        let mut ui = Ui::new(root);
        ui.set_viewport(viewport);
        let mut renderer = G::create_renderer(surface)?;
        graphics::Renderer::set_viewport(
            &mut renderer,
            (viewport.width * scale).round() as u32,
            (viewport.height * scale).round() as u32,
            scale,
        );
        let mut app = Self {
            ui,
            layout: LayoutState::new(engine),
            text: TextCache::new(G::create_text_system()?),
            tree: RenderTree::default(),
            renderer,
            scale,
            now: Duration::ZERO,
            last_surface: None,
            last_stats: RenderStats::default(),
        };
        app.frame()?;
        Ok(app)
    }

    /// The UI, e.g. to read the semantic tree.
    pub fn ui(&mut self) -> &mut Ui {
        &mut self.ui
    }

    /// The renderer, e.g. to read the last recorded frame.
    pub fn renderer(&self) -> &G::Renderer {
        &self.renderer
    }

    /// The renderer, mutably (e.g. to read back pixels).
    pub fn renderer_mut(&mut self) -> &mut G::Renderer {
        &mut self.renderer
    }

    /// The render tree.
    pub fn render_tree(&self) -> &RenderTree {
        &self.tree
    }

    /// The text cache, e.g. to register fonts.
    pub fn text_cache(&mut self) -> &mut TextCache<G::TextSystem> {
        &mut self.text
    }

    /// Current time on the manual clock.
    pub fn now(&self) -> Duration {
        self.now
    }

    /// The surface state committed most recently (materials).
    pub fn last_surface(&self) -> Option<&SurfaceState> {
        self.last_surface.as_ref()
    }

    /// Stats of the last frame.
    pub fn last_stats(&self) -> RenderStats {
        self.last_stats
    }

    /// Runs one complete frame at the current time.
    pub fn frame(&mut self) -> Result<RenderStats, GraphicsError> {
        if self.ui.wants_animation_frame() {
            self.ui.run_animation_frames(self.now);
        }
        let (batch, _cursor) =
            build_batch(&mut self.ui, &mut self.layout, &mut self.text, self.now);
        let (ops, surface, releases) = batch.upload(&mut self.renderer);
        if surface.is_some() {
            self.last_surface = surface;
        }
        self.tree.apply(ops, self.now, &mut self.renderer);
        self.tree.release_when_unused(releases);
        let (composition, stats) = self.tree.prepare(self.now, &mut self.renderer);
        graphics::Renderer::render(&mut self.renderer, &composition)?;
        self.last_stats = stats;
        Ok(stats)
    }

    /// Renders one frame with the clock at `now`, which never runs backwards: a host that
    /// owns the timeline (a compositor's presentation clock) drives animations with this.
    pub fn frame_at(&mut self, now: Duration) -> Result<RenderStats, GraphicsError> {
        self.now = self.now.max(now);
        self.frame()
    }

    /// Changes the logical viewport and the device scale; the next frame renders at the new size.
    pub fn resize(&mut self, viewport: Size, scale: f32) {
        self.scale = scale;
        self.ui.set_viewport(viewport);
        graphics::Renderer::set_viewport(
            &mut self.renderer,
            (viewport.width * scale).round() as u32,
            (viewport.height * scale).round() as u32,
            scale,
        );
    }

    /// Draws every rounded corner as `shape` from the next frame on.
    pub fn set_corner_shape(&mut self, shape: CornerShape) {
        graphics::Renderer::set_corner_shape(&mut self.renderer, shape);
        self.tree.record_all_again();
    }

    /// Whether another frame would differ from the last: something animates or changed.
    pub fn needs_frame(&self) -> bool {
        self.last_stats.animating || self.ui.needs_frame() || self.ui.wants_animation_frame()
    }

    /// Delivers a window event and renders a frame. A `Resized` size is in device pixels and
    /// keeps the current scale.
    pub fn handle(&mut self, event: &WindowEvent) -> Result<RenderStats, GraphicsError> {
        if let WindowEvent::Resized(size) = event {
            let logical = Size::new(
                size.width as f32 / self.scale,
                size.height as f32 / self.scale,
            );
            self.resize(logical, self.scale);
        }
        self.ui.handle_event(event);
        self.frame()
    }

    /// Performs a semantic action and renders a frame.
    pub fn perform(&mut self, id: SemanticId, action: &UiAction) -> Result<(), InlineError> {
        self.ui.perform(id, action)?;
        self.frame()?;
        Ok(())
    }

    /// Moves the clock forward by `elapsed` in frame-sized steps, rendering each frame.
    pub fn advance(&mut self, elapsed: Duration) -> Result<RenderStats, GraphicsError> {
        let target = self.now + elapsed;
        let mut stats = self.last_stats;
        while self.now < target {
            self.now = (self.now + FRAME_INTERVAL).min(target);
            stats = self.frame()?;
        }
        Ok(stats)
    }

    /// Advances until nothing animates (at most 10 simulated seconds).
    pub fn settle(&mut self) -> Result<RenderStats, GraphicsError> {
        let limit = self.now + SETTLE_LIMIT;
        let mut stats = self.frame()?;
        while (stats.animating || self.ui.needs_frame() || self.ui.wants_animation_frame())
            && self.now < limit
        {
            self.now += FRAME_INTERVAL;
            stats = self.frame()?;
        }
        Ok(stats)
    }
}

/// Why an inline action failed.
#[derive(Debug, thiserror::Error)]
pub enum InlineError {
    /// The UI refused the action.
    #[error(transparent)]
    Action(#[from] ActionError),
    /// Rendering failed.
    #[error(transparent)]
    Graphics(#[from] GraphicsError),
}
