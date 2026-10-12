use crownui::prelude::*;

fn main() -> Result<(), crownui::RunError> {
    tracing_subscriber::fmt::init();
    crownui::run(
        component(|cx| {
            let expanded = cx.signal(false);
            zstack((
                canvas(|_rt, size, list| {
                    use crownui::ext::{Brush, GradientStop, Paint};
                    let stops = [
                        (0.0, rgb(0x6366f1)),
                        (0.5, rgb(0xec4899)),
                        (1.0, rgb(0xf59e0b)),
                    ];
                    let gradient = Paint {
                        brush: Brush::LinearGradient {
                            start: Point::ZERO,
                            end: Point::new(size.width, size.height),
                            stops: stops
                                .iter()
                                .map(|(offset, color)| GradientStop {
                                    offset: *offset,
                                    color: *color,
                                })
                                .collect(),
                        },
                        blur: None,
                    };
                    list.fill(Rect::new(Point::ZERO, size), Default::default(), gradient);
                    for index in 0..6 {
                        let x = size.width * (0.15 + 0.14 * index as f32);
                        let y = size.height * (0.3 + 0.08 * (index % 3) as f32);
                        list.fill(
                            Rect::from_xywh(x, y, 60.0, 60.0),
                            CornerRadii::uniform(30.0),
                            rgba(0xffffff66),
                        );
                    }
                }),
                vstack((
                    vstack((
                        text("Glass").text_size(28.0).font_bold().text_color(WHITE),
                        text("Backdrop blur rendered by Impeller; springs run on the render thread.")
                            .text_color(rgba(0xffffffcc)),
                        show(
                            expanded,
                            || text("Expanded: the card grew with a smooth spring.").text_color(WHITE),
                            || text(""),
                        ),
                        button(bind(move |rt| if expanded.get(rt) { "Collapse".to_string() } else { "Expand".to_string() }))
                            .bg(rgba(0xffffff33))
                            .text_color(WHITE)
                            .hover(|style| style.bg(rgba(0xffffff55)))
                            .on_click(move |cx, _| {
                                cx.with_animation(Spring::bouncy(), |cx| expanded.update(cx, |open| *open = !*open))
                            }),
                    ))
                    .gap(12.0)
                    .p(24.0)
                    .w(bind(move |rt| if expanded.get(rt) { 420.0 } else { 340.0 }))
                    .rounded(24.0)
                    .bg(rgba(0xffffff26))
                    .border(1.0)
                    .border_color(rgba(0xffffff4d))
                    .backdrop_blur(24.0)
                    .shadow(Shadow::xl())
                    .transition(Spring::smooth()),
                ))
                .items_center()
                .justify_center(),
            ))
        }),
        AppConfig::new("Glass card").size(720, 520),
    )
}
