use geometry::Color;
use reactive::{Memo, Signal};
use semantics::Role;
use view::{Cx, Kind, Listeners, NodeId, Semantic, StyleProp, Styled, View, bind};
use windowing::CursorIcon;

use super::{
    date::{CELLS, CalendarDate},
    metrics::{
        ARROW_SIZE, ARROW_TARGET, CAPTION_TEXT, CHEVRON_LEFT, CHEVRON_RIGHT, DAY_TEXT, INSET_X,
        LINE_HEIGHT, OUTSIDE_MONTH_ALPHA, TITLE_HEIGHT, TITLE_TEXT, TODAY_DIAMETER, WEEK_HEIGHT,
        WEEKDAY_INITIALS, WEEKDAYS_HEIGHT,
    },
};
use crate::{
    kit::{INTER_FAMILY, Theme, icon::tinted_icon, use_theme},
    stack::{hstack, vstack},
    text::text,
    util::{color::scale_alpha, themed::themed},
};

/// The kind behind [`calendar`](super::calendar).
pub struct CalendarKind {
    pub(super) today: Signal<CalendarDate>,
}

fn styled_text(
    content: impl view::IntoProp<String>,
    (size, weight): (f32, u16),
) -> view::Element<crate::text::TextKind> {
    text(content)
        .text_size(size)
        .font_weight(weight)
        .leading(LINE_HEIGHT)
}

fn arrow(theme: Signal<Theme>, page: Signal<i32>, months: i32) -> impl View {
    let (glyph, label) = if months < 0 {
        (CHEVRON_LEFT, "Previous month")
    } else {
        (CHEVRON_RIGHT, "Next month")
    };
    hstack(
        tinted_icon(glyph, 2.0, themed(theme, |theme| theme.popover.muted_text)).size(ARROW_SIZE),
    )
    .h(ARROW_TARGET)
    .items_center()
    .justify_center()
    .cursor(CursorIcon::Pointer)
    .role(Role::Button)
    .label(label)
    .on_click(move |cx, _| page.update(cx, |page| *page += months))
}

fn day_color(theme: &Theme, day: CalendarDate, shown: CalendarDate, today: CalendarDate) -> Color {
    if day == today {
        theme.text.on_accent
    } else if day.month != shown.month {
        scale_alpha(theme.popover.muted_text, OUTSIDE_MONTH_ALPHA)
    } else if day.is_weekend() {
        theme.popover.muted_text
    } else {
        theme.popover.text
    }
}

fn day_cell(
    theme: Signal<Theme>,
    shown: Memo<CalendarDate>,
    today: Signal<CalendarDate>,
    cell: usize,
) -> impl View {
    let label = styled_text(
        bind(move |runtime| shown.get(runtime).grid_cell(cell).day.to_string()),
        DAY_TEXT,
    )
    .text_color(bind(move |runtime| {
        let (shown, today) = (shown.get(runtime), today.get(runtime));
        theme.with(runtime, |theme| {
            day_color(theme, shown.grid_cell(cell), shown, today)
        })
    }));
    let disc = hstack(label)
        .size(TODAY_DIAMETER)
        .rounded(TODAY_DIAMETER / 2.0)
        .items_center()
        .justify_center()
        .bg(bind(move |runtime| {
            let is_today = shown.get(runtime).grid_cell(cell) == today.get(runtime);
            theme.with(runtime, |theme| {
                if is_today {
                    theme.accent.end
                } else {
                    geometry::TRANSPARENT
                }
            })
        }));
    hstack(disc).h(WEEK_HEIGHT).items_center().justify_center()
}

impl Kind for CalendarKind {
    const NAME: &'static str = "kit_calendar";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let today = self.today;
        let page = cx.signal(0_i32);
        let shown =
            cx.memo(move |runtime| today.get(runtime).first_of_month_shifted(page.get(runtime)));

        cx.set_role(node, Role::Group);
        for prop in [
            StyleProp::FlexDirection(layouting::FlexDirection::Column),
            StyleProp::Padding(view::Edges::Horizontal, INSET_X),
            StyleProp::FontFamily(INTER_FAMILY.to_owned()),
        ] {
            cx.set_default_style(node, prop);
        }

        let title = hstack((
            styled_text(
                bind(move |runtime| shown.get(runtime).month_title()),
                TITLE_TEXT,
            )
            .role(Role::Heading)
            .text_color(themed(theme, |theme| theme.popover.text))
            .col_span(5),
            arrow(theme, page, -1),
            arrow(theme, page, 1),
        ))
        .grid()
        .grid_cols(7)
        .items_center()
        .h(TITLE_HEIGHT);
        let weekdays = hstack(
            WEEKDAY_INITIALS
                .iter()
                .map(|initial| {
                    styled_text(*initial, CAPTION_TEXT)
                        .text_center()
                        .text_color(themed(theme, |theme| theme.popover.muted_text))
                })
                .collect::<Vec<_>>(),
        )
        .grid()
        .grid_cols(7)
        .items_center()
        .h(WEEKDAYS_HEIGHT);
        let weeks = vstack(
            (0..CELLS)
                .map(|cell| day_cell(theme, shown, today, cell))
                .collect::<Vec<_>>(),
        )
        .grid()
        .grid_cols(7);
        cx.set_children(node, (title, weekdays, weeks));
    }
}
