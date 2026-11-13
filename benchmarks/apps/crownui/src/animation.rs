use std::time::Duration;

use crownui::prelude::*;

use crate::{
    config::AnimationConfig,
    geometry::{BoxGrid, LabelGrid},
    palette::{BACKGROUND, FOREGROUND, swatch},
};

pub fn view(config: AnimationConfig) -> impl View {
    component(move |cx| {
        let seconds = cx.signal(0.0_f32);
        tick(cx, seconds, None);
        let boxes = BoxGrid::new(config);
        let labels = LabelGrid::new(config, &boxes);
        let squares: Vec<_> = (0..config.boxes)
            .map(|index| square(index, boxes.position(index), boxes.box_size, seconds))
            .collect();
        let counters: Vec<_> = (0..config.labels)
            .map(|index| counter(index, labels.position(index), seconds))
            .collect();
        zstack((squares, counters))
            .w_full()
            .h_full()
            .bg(rgb(BACKGROUND))
    })
}

fn square(index: usize, (left, top): (f32, f32), size: f32, seconds: Signal<f32>) -> impl View {
    vstack(())
        .absolute()
        .left(left)
        .top(top)
        .size(size)
        .rounded(3.0)
        .bg(swatch(index))
        .translate(bind(move |rt| {
            let (dx, dy) = BoxGrid::offset(index, seconds.get(rt));
            Vector::new(dx, dy)
        }))
        .rotate(bind(move |rt| BoxGrid::rotation(index, seconds.get(rt))))
}

fn counter(index: usize, (left, top): (f32, f32), seconds: Signal<f32>) -> impl View {
    text(bind(move |rt| LabelGrid::text(index, seconds.get(rt))))
        .absolute()
        .left(left)
        .top(top)
        .text_size(11.0)
        .text_color(rgb(FOREGROUND))
}

fn tick(cx: &mut Cx, seconds: Signal<f32>, started: Option<Duration>) {
    cx.request_animation_frame(move |cx, now| {
        let started = started.unwrap_or(now);
        seconds.set(cx, (now - started).as_secs_f32());
        tick(cx, seconds, Some(started));
    });
}
