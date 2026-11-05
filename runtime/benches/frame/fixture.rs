use std::sync::mpsc;

use anyhow::{Context, Result};
use geometry::{Point, Rect, Size};
use graphics::RecordingBackend;
use runtime::InlineApp;
use semantics::Role;
use taffy_layout::TaffyEngine;
use view::component;
use widgets::scroll_view;

use crate::tree::{self, Family, Handles};

const VIEWPORT: Size = Size::new(1280.0, 800.0);
const SEED: u64 = 0x5EED_CAFE;

/// A mounted benchmark tree in a scroll view that fills the window, plus what the scenarios
/// need to poke at it.
pub(crate) struct Fixture {
    pub(crate) app: InlineApp<RecordingBackend, TaffyEngine>,
    pub(crate) handles: Handles,
    /// Centers of two buttons in view, to hover between.
    pub(crate) buttons: [Point; 2],
    pub(crate) viewport: Size,
    pub(crate) tick: u64,
}

impl Fixture {
    pub(crate) fn new(family: Family, size: usize) -> Result<Self> {
        let (sender, receiver) = mpsc::channel();
        let root = component(move |cx| {
            let (view, handles) = tree::build(cx, family, size, SEED);
            let _ = sender.send(handles);
            scroll_view(view)
        });
        let mut app = InlineApp::new(root, TaffyEngine::new(), VIEWPORT)?;
        let handles = receiver.recv().context("the tree was not built")?;
        let buttons = visible_buttons(&mut app);
        app.ui().take_counters();
        Ok(Self {
            app,
            handles,
            buttons,
            viewport: VIEWPORT,
            tick: 0,
        })
    }
}

fn visible_buttons(app: &mut InlineApp<RecordingBackend, TaffyEngine>) -> [Point; 2] {
    let window = Rect::new(Point::ZERO, VIEWPORT);
    let mut centers = app
        .ui()
        .semantic_tree()
        .iter()
        .filter(|node| node.role == Role::Button)
        .map(|node| node.bounds.center())
        .filter(|center| window.contains(*center));
    let first = centers.next().unwrap_or(Point::new(40.0, 20.0));
    let second = centers.next().unwrap_or(Point::new(40.0, 60.0));
    [first, second]
}
