use crowntest::{Role, TestApp, role, text as text_query};
use view::{Listeners, Styled, View, component};
use widgets::{
    kit::{Theme, button, provide_theme, slide_transition},
    text, vstack,
};

const PANE_WIDTH: f32 = 400.0;
const PANE_HEIGHT: f32 = 300.0;
const CONTAINER_WIDTH: f32 = 600.0;

fn deck() -> impl View {
    component(|cx| {
        provide_theme(cx, Theme::dark());
        let page = cx.signal(5usize);
        vstack((
            button("Next").on_click(move |cx, _| page.update(cx, |page| *page += 1)),
            button("Back").on_click(move |cx, _| page.update(cx, |page| *page -= 1)),
            slide_transition(page, |_, ordinal| {
                vstack(text(format!("Page {ordinal}")))
                    .w(PANE_WIDTH)
                    .h(PANE_HEIGHT)
            })
            .w(CONTAINER_WIDTH),
        ))
        .items_start()
    })
}

fn indent(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

fn pane_offset(frame: &str, label: &str) -> Option<f32> {
    let lines: Vec<&str> = frame.lines().collect();
    let target = format!("text \"{label}\"");
    let position = lines
        .iter()
        .position(|line| line.trim_start().starts_with(&target))?;
    let mut limit = indent(lines[position]);
    let mut offset = 0.0;
    for line in lines[..position].iter().rev() {
        if indent(line) >= limit {
            continue;
        }
        limit = indent(line);
        if let Some(origin) = line.trim_start().strip_prefix("layer at ") {
            offset += origin
                .split(',')
                .next()
                .and_then(|x| x.parse::<f32>().ok())?;
        }
    }
    Some(offset)
}

fn offsets(app: &TestApp, first: &str, second: &str) -> (Option<f32>, Option<f32>) {
    let frame = app.frame_snapshot();
    (pane_offset(&frame, first), pane_offset(&frame, second))
}

fn click(app: &mut TestApp, name: &str) {
    app.click(role(Role::Button).name(name));
}

#[test]
fn the_first_pane_is_simply_there_instead_of_sliding_in() {
    let app = TestApp::new(deck());
    assert!(!app.is_animating());
    assert_eq!(pane_offset(&app.frame_snapshot(), "Page 5"), Some(0.0));
}

#[test]
fn the_container_clips_to_the_incoming_panes_size() {
    let mut app = TestApp::new(deck());
    assert!(app.frame_snapshot().contains("clip 0,0 600x300"));
    app.settle();
    click(&mut app, "Next");
    app.settle();
    assert!(app.frame_snapshot().contains("clip 0,0 600x300"));
}

#[test]
fn going_forward_the_new_pane_waits_a_container_width_to_the_right() {
    let mut app = TestApp::new(deck());
    click(&mut app, "Next");
    assert_eq!(
        offsets(&app, "Page 6", "Page 5"),
        (Some(CONTAINER_WIDTH), Some(0.0))
    );
}

#[test]
fn going_back_the_new_pane_waits_a_container_width_to_the_left() {
    let mut app = TestApp::new(deck());
    click(&mut app, "Back");
    assert_eq!(
        offsets(&app, "Page 4", "Page 5"),
        (Some(-CONTAINER_WIDTH), Some(0.0))
    );
}

#[test]
fn both_panes_travel_together_one_container_width_apart() {
    let mut app = TestApp::new(deck());
    click(&mut app, "Next");
    app.advance_ms(96);
    let (Some(incoming), Some(outgoing)) = offsets(&app, "Page 6", "Page 5") else {
        panic!(
            "both panes should paint mid-slide:\n{}",
            app.frame_snapshot()
        );
    };
    assert!(incoming > 0.0 && incoming < CONTAINER_WIDTH, "{incoming}");
    assert!(outgoing < 0.0 && outgoing > -CONTAINER_WIDTH, "{outgoing}");
    assert!((incoming - outgoing - CONTAINER_WIDTH).abs() < 0.05);
    app.advance_ms(96);
    let (later, _) = offsets(&app, "Page 6", "Page 5");
    assert!(later.is_some_and(|later| (0.0..incoming).contains(&later)));
}

#[test]
fn the_slide_settles_with_the_new_pane_at_rest_and_the_old_one_retired() {
    let mut app = TestApp::new(deck());
    click(&mut app, "Next");
    app.advance_ms(16);
    assert!(app.is_animating());
    app.settle();
    app.advance_ms(16);
    assert!(!app.is_animating());
    assert_eq!(
        offsets(&app, "Page 6", "Page 5"),
        (Some(0.0), None),
        "{}",
        app.frame_snapshot()
    );
}

#[test]
fn the_slide_rests_within_a_second_and_a_half() {
    let mut app = TestApp::new(deck());
    click(&mut app, "Next");
    app.advance_ms(1400);
    assert!(!app.is_animating());
}

#[test]
fn only_the_current_pane_is_in_the_accessibility_tree() {
    let mut app = TestApp::new(deck());
    click(&mut app, "Next");
    assert!(app.exists(text_query("Page 6")));
    assert!(!app.exists(text_query("Page 5")));
}

#[test]
fn interrupting_a_slide_hands_the_moving_pane_over_smoothly() {
    let mut app = TestApp::new(deck());
    click(&mut app, "Next");
    app.advance_ms(64);
    let (Some(entering), _) = offsets(&app, "Page 6", "Page 5") else {
        panic!("the incoming pane should be mid-slide");
    };
    click(&mut app, "Next");
    assert_eq!(
        offsets(&app, "Page 7", "Page 6"),
        (Some(CONTAINER_WIDTH), Some(entering))
    );
    app.settle();
    app.advance_ms(16);
    assert_eq!(offsets(&app, "Page 7", "Page 6"), (Some(0.0), None));
}
