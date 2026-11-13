use std::time::Instant;

use gpui::{
    Context, Div, IntoElement, Render, ScrollHandle, Stateful, Window, div, point, prelude::*, px,
    relative, rgb, svg,
};

use crate::{
    config::SettingsConfig,
    palette::{ACCENT, BACKGROUND, FOREGROUND, MUTED},
    settings::{
        Control, control, open_page, page_icon, page_title, row_description, row_index, row_title,
        scroll_offset, section_title,
    },
};

const SIDEBAR_BACKGROUND: u32 = 0xeef2f7;
const SELECTED: u32 = 0xdde4ee;
const CARD: u32 = 0xffffff;
const BORDER: u32 = 0xe2e8f0;
const TRACK: u32 = 0xcbd5e1;
const CHEVRON: &str = "icons/chevron-right.svg";

pub struct SettingsWindow {
    config: SettingsConfig,
    scroll: ScrollHandle,
    started: Instant,
    page: Option<usize>,
}

impl SettingsWindow {
    pub fn new(config: SettingsConfig) -> Self {
        Self {
            config,
            scroll: ScrollHandle::new(),
            started: Instant::now(),
            page: None,
        }
    }
}

impl Render for SettingsWindow {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let config = self.config;
        let (page, open_for) = open_page(config, self.started.elapsed());
        if self.page != Some(page) {
            self.page = Some(page);
            self.scroll = ScrollHandle::new();
        }
        let travel = f32::from(self.scroll.max_offset().height);
        let offset = scroll_offset(config, open_for, travel);
        self.scroll.set_offset(point(px(0.0), px(-offset)));
        window.request_animation_frame();
        div()
            .flex()
            .size_full()
            .p(px(6.0))
            .bg(rgb(SIDEBAR_BACKGROUND))
            .child(sidebar(config, page))
            .child(
                div()
                    .flex_1()
                    .rounded(px(12.0))
                    .bg(rgb(BACKGROUND))
                    .overflow_hidden()
                    .child(page_view(config, page, &self.scroll)),
            )
    }
}

fn sidebar(config: SettingsConfig, open: usize) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(2.0))
        .w(px(config.sidebar_width))
        .p(px(8.0))
        .child(
            div()
                .px(px(8.0))
                .py(px(6.0))
                .text_size(px(11.0))
                .text_color(rgb(MUTED))
                .child("Settings"),
        )
        .children((0..config.pages).map(|page| {
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .px(px(8.0))
                .h(px(32.0))
                .rounded(px(8.0))
                .when(page == open, |item| item.bg(rgb(SELECTED)))
                .child(icon(&format!("icons/{}.svg", page_icon(page))))
                .child(
                    div()
                        .text_size(px(13.0))
                        .text_color(rgb(FOREGROUND))
                        .child(page_title(page)),
                )
        }))
}

fn page_view(config: SettingsConfig, page: usize, scroll: &ScrollHandle) -> Stateful<Div> {
    div()
        .id(("page", page))
        .size_full()
        .overflow_y_scroll()
        .track_scroll(scroll)
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(24.0))
                .p(px(32.0))
                .child(
                    div()
                        .text_size(px(22.0))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgb(FOREGROUND))
                        .child(page_title(page)),
                )
                .children((0..config.sections).map(|number| section(config, page, number))),
        )
}

fn section(config: SettingsConfig, page: usize, number: usize) -> Div {
    let rows = (0..config.rows).map(|row| {
        let index = row_index(config, number, row);
        row_view(page, index).when(row > 0, |row| row.border_t_1().border_color(rgb(BORDER)))
    });
    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(
            div()
                .px(px(4.0))
                .text_size(px(12.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(rgb(MUTED))
                .child(section_title(number)),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .rounded(px(12.0))
                .border_1()
                .border_color(rgb(BORDER))
                .bg(rgb(CARD))
                .children(rows),
        )
}

fn row_view(page: usize, index: usize) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(12.0))
        .px(px(18.0))
        .py(px(14.0))
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .gap(px(3.0))
                .child(
                    div()
                        .text_size(px(14.0))
                        .text_color(rgb(FOREGROUND))
                        .child(row_title(page, index)),
                )
                .child(
                    div()
                        .text_size(px(12.0))
                        .text_color(rgb(MUTED))
                        .child(row_description(index)),
                ),
        )
        .child(control_view(control(page, index)))
}

fn control_view(control: Control) -> Div {
    match control {
        Control::Switch(on) => div()
            .flex()
            .items_center()
            .w(px(36.0))
            .h(px(20.0))
            .p(px(2.0))
            .rounded_full()
            .bg(rgb(if on { ACCENT } else { TRACK }))
            .when(on, |track| track.justify_end())
            .child(div().size(px(16.0)).rounded_full().bg(rgb(CARD))),
        Control::Slider(value) => div()
            .relative()
            .flex()
            .items_center()
            .w(px(160.0))
            .h(px(16.0))
            .child(div().w_full().h(px(4.0)).rounded_full().bg(rgb(TRACK)))
            .child(
                div()
                    .absolute()
                    .left_0()
                    .w(relative(value))
                    .h(px(4.0))
                    .rounded_full()
                    .bg(rgb(ACCENT)),
            )
            .child(
                div()
                    .absolute()
                    .left(px(value * 144.0))
                    .size(px(16.0))
                    .rounded_full()
                    .border_1()
                    .border_color(rgb(BORDER))
                    .bg(rgb(CARD)),
            ),
        Control::Value(value) => div()
            .flex()
            .items_center()
            .gap(px(4.0))
            .child(
                div()
                    .text_size(px(14.0))
                    .text_color(rgb(MUTED))
                    .child(value),
            )
            .child(icon(CHEVRON)),
        Control::Chevron => div().child(icon(CHEVRON)),
    }
}

fn icon(path: &str) -> gpui::Svg {
    svg()
        .path(path.to_owned())
        .size(px(16.0))
        .text_color(rgb(MUTED))
}
