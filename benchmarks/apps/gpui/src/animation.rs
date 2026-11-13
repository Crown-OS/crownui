use std::time::Instant;

use gpui::{
    Context, IntoElement, Render, Transformation, Window, div, prelude::*, px, radians, rgb, svg,
};

use crate::{
    assets::SQUARE,
    config::AnimationConfig,
    geometry::{BoxGrid, LabelGrid},
    palette::{BACKGROUND, FOREGROUND, swatch},
};

pub struct Animation {
    config: AnimationConfig,
    boxes: BoxGrid,
    labels: LabelGrid,
    started: Instant,
}

impl Animation {
    pub fn new(config: AnimationConfig) -> Self {
        let boxes = BoxGrid::new(config);
        let labels = LabelGrid::new(config, &boxes);
        Self {
            config,
            boxes,
            labels,
            started: Instant::now(),
        }
    }
}

impl Render for Animation {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let seconds = self.started.elapsed().as_secs_f32();
        window.request_animation_frame();
        let squares = (0..self.config.boxes).map(|index| {
            let (left, top) = self.boxes.position(index);
            let (dx, dy) = BoxGrid::offset(index, seconds);
            svg()
                .path(SQUARE)
                .absolute()
                .left(px(left + dx))
                .top(px(top + dy))
                .size(px(self.boxes.box_size))
                .text_color(swatch(index))
                .with_transformation(Transformation::rotate(radians(
                    BoxGrid::rotation(index, seconds).to_radians(),
                )))
        });
        let counters = (0..self.config.labels).map(|index| {
            let (left, top) = self.labels.position(index);
            div()
                .absolute()
                .left(px(left))
                .top(px(top))
                .text_size(px(11.0))
                .text_color(rgb(FOREGROUND))
                .child(LabelGrid::text(index, seconds))
        });
        div()
            .relative()
            .size_full()
            .bg(rgb(BACKGROUND))
            .children(squares)
            .children(counters)
    }
}
