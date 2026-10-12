use graphics::TextSystem;
use layouting::LayoutEngine;
use view::{LayoutState, TextCache, Ui};

use crate::batch::RenderBatch;

/// Runs the UI-thread half of a frame: reactive updates, layout and render-op collection.
pub(crate) fn build_batch<L: LayoutEngine, T: TextSystem>(
    ui: &mut Ui,
    layout: &mut LayoutState<L>,
    text: &mut TextCache<T>,
    now: std::time::Duration,
) -> (RenderBatch<T::Layout>, Option<windowing::CursorIcon>) {
    ui.update(now);
    ui.layout(layout, text);
    let frame = ui.take_frame();
    let batch = RenderBatch {
        ops: frame.ops,
        text_uploads: text.take_uploads(),
        text_releases: text.take_releases(),
        images: frame.images,
        image_releases: frame.image_releases,
        surface: frame.surface,
    };
    (batch, frame.cursor)
}
