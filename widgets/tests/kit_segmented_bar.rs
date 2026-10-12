use crowntest::{Role, TestApp, role};
use geometry::rgb;
use view::{Listeners, Styled, component};
use widgets::{
    kit::{Segment, SegmentedBarOptions, Theme, button, provide_theme, segmented_bar},
    vstack,
};

const RED: geometry::Color = rgb(0xff0000);
const GREEN: geometry::Color = rgb(0x00ff00);
const BLUE: geometry::Color = rgb(0x0000ff);

fn stage(content: impl view::ViewSeq) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, Theme::dark());
        vstack(content).p(24.0).w(348.0)
    }))
}

fn runs(app: &mut TestApp) -> Vec<String> {
    app.frame_snapshot()
        .lines()
        .filter(|line| line.trim().starts_with("fill"))
        .map(|line| line.trim().to_owned())
        .collect()
}

#[test]
fn the_bar_fills_the_column_and_is_fourteen_pixels_thick() {
    let mut app = stage(segmented_bar([Segment::new(1.0, RED)]));
    assert!(
        app.layout_snapshot()
            .contains("kit_segmented_bar @24,24 300x14")
    );
}

#[test]
fn the_thickness_is_overridable() {
    let mut app = stage(segmented_bar([Segment::new(1.0, RED)]).h(20.0));
    assert!(
        app.layout_snapshot()
            .contains("kit_segmented_bar @24,24 300x20")
    );
}

#[test]
fn runs_are_proportional_to_their_weights() {
    let mut app = stage(segmented_bar([
        Segment::new(1.0, RED),
        Segment::new(2.0, GREEN),
        Segment::new(3.0, BLUE),
    ]));
    assert_eq!(
        runs(&mut app),
        [
            "fill 0,0 50x14 #ff0000ff",
            "fill 50,0 100x14 #00ff00ff",
            "fill 150,0 150x14 #0000ffff"
        ]
    );
}

#[test]
fn the_runs_are_clipped_to_one_pill_not_rounded_individually() {
    let mut app = stage(segmented_bar([
        Segment::new(1.0, RED),
        Segment::new(1.0, BLUE),
    ]));
    let frame = app.frame_snapshot();
    assert!(frame.contains("clip 0,0 300x14 r7"), "{frame}");
    assert!(runs(&mut app).iter().all(|run| !run.contains(" r")));
}

#[test]
fn the_final_run_reaches_the_right_edge() {
    let mut app = stage(segmented_bar([
        Segment::new(1.0, RED),
        Segment::new(1.0, GREEN),
        Segment::new(1.0, BLUE),
    ]));
    assert_eq!(
        runs(&mut app).last().map(String::as_str),
        Some("fill 200,0 100x14 #0000ffff")
    );
}

#[test]
fn a_zero_weight_run_takes_no_space() {
    let mut app = stage(segmented_bar([
        Segment::new(1.0, RED),
        Segment::new(0.0, GREEN),
        Segment::new(1.0, BLUE),
    ]));
    let runs = runs(&mut app);
    assert_eq!(runs.len(), 2, "{runs:?}");
    assert_eq!(runs[0], "fill 0,0 150x14 #ff0000ff");
}

#[test]
fn nonsense_weights_contribute_nothing() {
    let mut app = stage(segmented_bar([
        Segment::new(1.0, RED),
        Segment::new(-5.0, GREEN),
        Segment::new(f32::NAN, GREEN),
        Segment::new(1.0, BLUE),
    ]));
    let runs = runs(&mut app);
    assert_eq!(runs[0], "fill 0,0 150x14 #ff0000ff");
    assert_eq!(
        runs.last().map(String::as_str),
        Some("fill 150,0 150x14 #0000ffff")
    );
    assert!(
        !runs.iter().any(|run| run.contains("#00ff00ff")),
        "{runs:?}"
    );
}

#[test]
fn a_zero_total_paints_the_empty_track_pill() {
    for segments in [vec![], vec![Segment::new(0.0, RED)]] {
        let mut app = stage(segmented_bar(segments));
        assert_eq!(runs(&mut app), ["fill 0,0 300x14 r7 #3a3a3dff"]);
    }
}

#[test]
fn live_segments_follow_their_signals() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let used = cx.signal(1.0_f32);
        vstack((
            segmented_bar([]).segments_from(move |runtime| {
                vec![
                    Segment::new(used.get(runtime), RED),
                    Segment::new(1.0, BLUE),
                ]
            }),
            button("More").on_click(move |cx, _| used.set(cx, 3.0)),
        ))
        .p(24.0)
        .w(348.0)
    }));
    assert_eq!(runs(&mut app)[0], "fill 0,0 150x14 #ff0000ff");
    app.click(role(Role::Button).name("More"));
    assert_eq!(runs(&mut app)[0], "fill 0,0 225x14 #ff0000ff");
}

#[test]
fn an_empty_bar_follows_the_theme_track() {
    let mut app = TestApp::new(component(|cx| {
        let theme = provide_theme(cx, Theme::light());
        vstack((
            segmented_bar([]),
            button("Dark").on_click(move |cx, _| theme.set(cx, Theme::dark())),
        ))
        .p(24.0)
        .w(348.0)
    }));
    assert!(app.frame_snapshot().contains("#e7e5f1ff"));
    app.click(role(Role::Button).name("Dark"));
    assert!(app.frame_snapshot().contains("#3a3a3dff"));
}

#[test]
fn a_bar_composes_with_a_legend_column() {
    let mut app = stage((
        segmented_bar([
            Segment::new(120.0, rgb(0x9f9f9f)),
            Segment::new(80.0, rgb(0x6d48e8)),
            Segment::new(300.0, rgb(0x3a3a3d)),
        ]),
        segmented_bar([]),
    ));
    insta::assert_snapshot!(app.snapshot());
}
