use geometry::{CornerRadii, Point, Rect};
use reactive::Signal;
use view::{Styled, View, component};

use crate::{
    canvas::canvas,
    kit::{
        MenuRowOptions, Theme, context_menu, icons, menu_item, menu_item_selected, menu_separator,
        menu_submenu, preview::Preview, provide_theme,
    },
    stack::vstack,
};

const STREAKS: [(Rect, f32); 3] = [
    (Rect::from_xywh(-40.0, 40.0, 560.0, 28.0), 0.7),
    (Rect::from_xywh(160.0, 140.0, 400.0, 48.0), 0.45),
    (Rect::from_xywh(-20.0, 250.0, 380.0, 20.0), 0.6),
];

fn wallpaper(theme: Signal<Theme>) -> impl View {
    canvas(move |runtime, size, list| {
        let theme = theme.get(runtime);
        let bounds = Rect::new(Point::ZERO, size);
        list.fill(
            bounds,
            CornerRadii::ZERO,
            theme.accent.horizontal(0.0, size.width),
        );
        for (streak, strength) in STREAKS {
            let radii = CornerRadii::uniform(streak.size.height / 2.0);
            list.fill(
                streak,
                radii,
                theme.surface.raised.with_alpha_factor(strength),
            );
        }
    })
    .absolute()
    .inset(0.0)
}

fn finder_menu() -> impl View {
    context_menu((
        menu_item("Open").leading_icon(icons::SQUARE_ARROW_UP_RIGHT),
        menu_item("Remove Download").leading_icon(icons::CIRCLE_X),
        menu_item("Keep Downloaded").leading_icon(icons::CIRCLE_ARROW_DOWN),
        menu_separator(),
        menu_item("Move to Trash").leading_icon(icons::TRASH),
        menu_separator(),
        menu_item("Get Info").leading_icon(icons::INFO),
        menu_item_selected("Rename").leading_icon(icons::PEN_LINE),
        menu_item("Duplicate").leading_icon(icons::COPY_PLUS),
        menu_separator(),
        menu_submenu("Import from iPhone"),
        menu_submenu("Quick Actions"),
    ))
    .absolute()
    .left(150.0)
    .top(16.0)
}

pub(super) fn previews() -> Vec<Preview> {
    vec![Preview {
        name: "context_menu_glass",
        build: || {
            component(|cx| {
                let theme = provide_theme(cx, Theme::light());
                vstack((wallpaper(theme), finder_menu())).w(480.0).h(320.0)
            })
            .boxed()
        },
    }]
}
