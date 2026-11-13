use std::time::Duration;

use crownui::prelude::*;

use crate::{
    config::ListConfig,
    palette::{BACKGROUND, FOREGROUND, MUTED, swatch},
};

pub fn view(config: ListConfig) -> impl View {
    component(move |cx| {
        let rows = cx.signal(config.rows);
        let revision = cx.signal(0_u64);
        let handle = ScrollHandle::new(cx);
        scroll_continuously(cx, handle, config, None);
        lazy_list(rows, revision, config.row_height, move |_, index| {
            row(index, config)
        })
        .scroll_handle(handle)
        .w_full()
        .h_full()
        .bg(rgb(BACKGROUND))
    })
}

fn row(index: usize, config: ListConfig) -> impl View {
    hstack((
        vstack(())
            .size(config.swatch)
            .rounded(6.0)
            .bg(swatch(index)),
        vstack((
            text(format!("Row {index}"))
                .text_size(15.0)
                .font_medium()
                .text_color(rgb(FOREGROUND)),
            text(format!("Subtitle for row {index}"))
                .text_size(12.0)
                .text_color(rgb(MUTED)),
        ))
        .gap(2.0),
    ))
    .gap(12.0)
    .px(16.0)
    .h(config.row_height)
    .items_center()
}

/// Moves the list at a constant speed from the first frame on, wrapping at the end.
fn scroll_continuously(
    cx: &mut Cx,
    handle: ScrollHandle,
    config: ListConfig,
    started: Option<Duration>,
) {
    cx.request_animation_frame(move |cx, now| {
        let started = started.unwrap_or(now);
        let travel = (config.rows as f32 * config.row_height - config.viewport_height).max(1.0);
        let offset = (now - started).as_secs_f32() * config.scroll_speed % travel;
        handle.scroll_to_offset(cx, Vector::new(0.0, offset));
        scroll_continuously(cx, handle, config, Some(started));
    });
}
