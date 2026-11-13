use std::time::Duration;

use crownui::kit::{
    INTER_FAMILY, Theme, divider, icon, icons, section, sidebar, sidebar_item, sidebar_section,
    slider, toggle, use_theme,
};
use crownui::prelude::*;

use crate::{
    config::SettingsConfig,
    settings::{
        Control, control, open_page, page_icon, page_title, row_description, row_index, row_title,
        scroll_offset, section_title,
    },
};

const PAGE_PADDING: f32 = 32.0;
const CARD_GAP: f32 = 24.0;
const ROW_PADDING_V: f32 = 14.0;
const ROW_PADDING_H: f32 = 18.0;
const ROW_DESCRIPTION_GAP: f32 = 3.0;
const ROW_CONTROL_GAP: f32 = 12.0;
const TITLE_SIZE: f32 = 22.0;
const ROW_TITLE_SIZE: f32 = 14.0;
const ROW_DESCRIPTION_SIZE: f32 = 12.0;
const SLIDER_WIDTH: f32 = 160.0;

pub fn view(config: SettingsConfig) -> impl View {
    component(move |cx| {
        let page = cx.signal(0_usize);
        follow_pages(cx, page, config, None);
        let items: Vec<_> = (0..config.pages)
            .map(|index| {
                sidebar_item(
                    page_title(index),
                    lucide(index),
                    bind(move |rt| page.get(rt) == index),
                )
            })
            .collect();
        let menus = sidebar_section("Settings", items).w(config.sidebar_width);
        let content = keyed(
            move |rt| page.get(rt),
            move |_, page| page_view(page, config),
        )
        .flex_1();
        sidebar(menus, content)
    })
}

/// Opens the page the scenario's clock says is open, from the first frame on.
fn follow_pages(
    cx: &mut Cx,
    page: Signal<usize>,
    config: SettingsConfig,
    started: Option<Duration>,
) {
    cx.request_animation_frame(move |cx, now| {
        let started = started.unwrap_or(now);
        let (open, _) = open_page(config, now - started);
        if page.get(cx) != open {
            page.set(cx, open);
        }
        follow_pages(cx, page, config, Some(started));
    });
}

fn page_view(page: usize, config: SettingsConfig) -> impl View {
    component(move |cx| {
        let theme = use_theme(cx);
        let handle = ScrollHandle::new(cx);
        scroll_page(cx, handle, config, None);
        let sections: Vec<_> = (0..config.sections)
            .map(|index| section_view(theme, page, index, config))
            .collect();
        scroll_view(
            vstack((
                text(page_title(page))
                    .text_size(TITLE_SIZE)
                    .font_semibold()
                    .text_color(tint(theme, |theme| theme.text.primary)),
                sections,
            ))
            .gap(CARD_GAP)
            .p(PAGE_PADDING),
        )
        .scroll_handle(handle)
        .flex_1()
        .font_family(INTER_FAMILY)
    })
}

/// Scrolls the page at the scenario's speed from its first frame, wrapping at the end.
fn scroll_page(
    cx: &mut Cx,
    handle: ScrollHandle,
    config: SettingsConfig,
    opened: Option<Duration>,
) {
    cx.request_animation_frame(move |cx, now| {
        let opened = opened.unwrap_or(now);
        let travel = handle.content_size().get(cx).height - handle.size().get(cx).height;
        let offset = scroll_offset(config, (now - opened).as_secs_f32(), travel);
        handle.scroll_to_offset(cx, Vector::new(0.0, offset));
        scroll_page(cx, handle, config, Some(opened));
    });
}

fn section_view(
    theme: Signal<Theme>,
    page: usize,
    number: usize,
    config: SettingsConfig,
) -> impl View {
    let rows: Vec<AnyView> = (0..config.rows)
        .flat_map(|row| {
            let index = row_index(config, number, row);
            let divider = (row > 0).then(|| divider().boxed());
            divider
                .into_iter()
                .chain([row_view(theme, page, index).boxed()])
        })
        .collect();
    section(section_title(number), rows)
}

fn row_view(theme: Signal<Theme>, page: usize, index: usize) -> impl View {
    component(move |cx| {
        let label = vstack((
            text(row_title(page, index))
                .text_size(ROW_TITLE_SIZE)
                .text_color(tint(theme, |theme| theme.text.body)),
            text(row_description(index))
                .text_size(ROW_DESCRIPTION_SIZE)
                .text_color(tint(theme, |theme| theme.text.muted)),
        ))
        .gap(ROW_DESCRIPTION_GAP)
        .shrink(1.0);
        hstack((
            label,
            spacer(),
            control_view(cx, theme, control(page, index)),
        ))
        .items_center()
        .gap(ROW_CONTROL_GAP)
        .px(ROW_PADDING_H)
        .py(ROW_PADDING_V)
    })
}

fn control_view(cx: &mut Cx, theme: Signal<Theme>, control: Control) -> AnyView {
    match control {
        Control::Switch(on) => toggle(cx.signal(on)).boxed(),
        Control::Slider(value) => slider(cx.signal(value)).w(SLIDER_WIDTH).boxed(),
        Control::Value(value) => hstack((
            text(value)
                .text_size(ROW_TITLE_SIZE)
                .text_color(tint(theme, |theme| theme.text.muted)),
            icon(icons::CHEVRON_RIGHT),
        ))
        .items_center()
        .gap(4.0)
        .boxed(),
        Control::Chevron => icon(icons::CHEVRON_RIGHT).boxed(),
    }
}

/// The kit's Lucide body for a page's icon file.
fn lucide(page: usize) -> &'static str {
    match page_icon(page) {
        "wifi" => icons::WIFI,
        "bluetooth" => icons::BLUETOOTH,
        "globe" => icons::GLOBE,
        "monitor" => icons::MONITOR,
        "volume-2" => icons::VOLUME_2,
        "bell" => icons::BELL,
        "battery-full" => icons::BATTERY_FULL,
        _ => icons::PALETTE,
    }
}

fn tint(theme: Signal<Theme>, pick: impl Fn(&Theme) -> Color + 'static) -> impl IntoProp<Color> {
    bind(move |rt| theme.with(rt, &pick))
}
