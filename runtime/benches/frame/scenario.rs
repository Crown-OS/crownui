use anyhow::Result;
use geometry::{Point, Size, Vector};
use input::{Modifiers, PointerEvent, WheelDelta, WheelEvent};
use runtime::RenderStats;
use view::UiCounters;
use windowing::WindowEvent;

use crate::fixture::Fixture;

const SCROLL_STEP: f32 = 40.0;

/// One kind of change, applied as one event followed by the frame it causes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Scenario {
    /// Nothing changed; what the UI thread does to find that out.
    Unchanged,
    /// The pointer moves between two buttons with hover styles.
    Hover,
    /// One wheel tick on the scroll view.
    ScrollTick,
    /// One leaf's text signal changes.
    TextChange,
    /// A row is appended, then removed.
    InsertRemoveRows,
    /// The window grows and shrinks by one pixel.
    RootResize,
}

impl Scenario {
    pub(crate) const ALL: [Scenario; 6] = [
        Scenario::Unchanged,
        Scenario::Hover,
        Scenario::ScrollTick,
        Scenario::TextChange,
        Scenario::InsertRemoveRows,
        Scenario::RootResize,
    ];

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Unchanged => "unchanged",
            Self::Hover => "hover",
            Self::ScrollTick => "scroll_tick",
            Self::TextChange => "text_change",
            Self::InsertRemoveRows => "insert_remove_rows",
            Self::RootResize => "root_resize",
        }
    }

    /// Applies the change and renders; returns what the UI thread and the render tree did.
    pub(crate) fn step(self, fixture: &mut Fixture) -> Result<(UiCounters, RenderStats)> {
        fixture.tick += 1;
        let odd = fixture.tick % 2 == 1;
        let app = &mut fixture.app;
        let stats = match self {
            Self::Unchanged => {
                if app.needs_frame() {
                    app.frame()?
                } else {
                    RenderStats::default()
                }
            }
            Self::Hover => {
                let target = fixture.buttons[usize::from(odd)];
                app.handle(&WindowEvent::PointerMove(PointerEvent::mouse(target)))?
            }
            Self::ScrollTick => {
                let step = if odd { SCROLL_STEP } else { -SCROLL_STEP };
                app.handle(&WindowEvent::Wheel(WheelEvent {
                    delta: WheelDelta::Pixels(Vector::new(0.0, step)),
                    position: center(fixture.viewport),
                    modifiers: Modifiers::NONE,
                }))?
            }
            Self::TextChange => {
                let text = format!("Row {}", fixture.tick);
                fixture.handles.text.set(app.ui().cx(), text);
                app.frame()?
            }
            Self::InsertRemoveRows => {
                let added = fixture.tick as u32;
                fixture.handles.rows.update(app.ui().cx(), |rows| {
                    if odd {
                        rows.push(added);
                    } else {
                        rows.pop();
                    }
                });
                app.frame()?
            }
            Self::RootResize => {
                let width = fixture.viewport.width + if odd { 1.0 } else { 0.0 };
                app.resize(Size::new(width, fixture.viewport.height), 1.0);
                app.frame()?
            }
        };
        Ok((app.ui().take_counters(), stats))
    }
}

fn center(size: Size) -> Point {
    Point::new(size.width / 2.0, size.height / 2.0)
}
