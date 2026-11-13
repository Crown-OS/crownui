use std::{ops::Range, time::Instant};

use gpui::{
    Context, Div, IntoElement, Render, UniformListScrollHandle, Window, div, point, prelude::*, px,
    rgb, uniform_list,
};

use crate::{
    config::ListConfig,
    palette::{BACKGROUND, FOREGROUND, MUTED, swatch},
};

pub struct List {
    config: ListConfig,
    scroll: UniformListScrollHandle,
    started: Instant,
}

impl List {
    pub fn new(config: ListConfig) -> Self {
        Self {
            config,
            scroll: UniformListScrollHandle::new(),
            started: Instant::now(),
        }
    }

    fn rows(&mut self, range: Range<usize>) -> Vec<Div> {
        range.map(|index| row(index, self.config)).collect()
    }
}

impl Render for List {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let config = self.config;
        let travel = (config.rows as f32 * config.row_height - config.viewport_height).max(1.0);
        let offset = self.started.elapsed().as_secs_f32() * config.scroll_speed % travel;
        self.scroll
            .0
            .borrow()
            .base_handle
            .set_offset(point(px(0.0), px(-offset)));
        window.request_animation_frame();
        uniform_list(
            "rows",
            config.rows,
            cx.processor(|list, range, _window, _cx| list.rows(range)),
        )
        .track_scroll(self.scroll.clone())
        .size_full()
        .bg(rgb(BACKGROUND))
    }
}

fn row(index: usize, config: ListConfig) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(12.0))
        .px(px(16.0))
        .h(px(config.row_height))
        .child(
            div()
                .size(px(config.swatch))
                .rounded(px(6.0))
                .bg(swatch(index)),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .text_size(px(15.0))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(rgb(FOREGROUND))
                        .child(format!("Row {index}")),
                )
                .child(
                    div()
                        .text_size(px(12.0))
                        .text_color(rgb(MUTED))
                        .child(format!("Subtitle for row {index}")),
                ),
        )
}
