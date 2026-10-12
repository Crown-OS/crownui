use crowntest::{Role, TestApp, role, text as text_query};
use geometry::Color;
use view::{Listeners, Prop, Styled, View, component};
use widgets::{
    kit::{
        ButtonOptions, ButtonVariant, Theme, button, provide_theme, wizard, wizard_footer,
        wizard_page, wizard_rail, wizard_scroll,
    },
    text, vstack,
};

const STEPS: [&str; 4] = ["Welcome", "Disk", "Review", "Restart"];
const RING: &str = "stroke2 1,1 31x31 r15.5";

fn hex(color: Color) -> String {
    let [red, green, blue, alpha] = color.to_rgba8();
    format!("#{red:02x}{green:02x}{blue:02x}{alpha:02x}")
}

fn flow(theme: Theme, start: usize) -> impl View {
    component(move |cx| {
        provide_theme(cx, theme);
        let step = cx.signal(start);
        wizard(
            STEPS,
            step,
            wizard_page(
                "How much room?",
                Some("Windows keeps everything."),
                wizard_scroll(vstack(text("controls"))),
            ),
            wizard_footer(
                button("Back").on_click(move |cx, _| step.update(cx, |step| *step -= 1)),
                button("Continue")
                    .variant(ButtonVariant::Primary)
                    .on_click(move |cx, _| step.update(cx, |step| *step += 1)),
            ),
        )
    })
}

fn rail_only(theme: Theme, steps: Vec<&'static str>, current: usize) -> impl View {
    component(move |cx| {
        provide_theme(cx, theme);
        vstack(wizard_rail(steps, Prop::Static(current))).w(400.0)
    })
}

fn assert_close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.01, "{actual} vs {expected}");
}

fn checks(frame: &str) -> usize {
    frame.matches("stroke2 path[3] 4,6 16x11").count()
}

#[test]
fn the_rail_spans_its_container_and_stacks_dots_over_labels() {
    let mut app = TestApp::new(flow(Theme::dark(), 1));
    let layout = app.layout_snapshot();
    assert!(
        layout.contains("kit_wizard_rail @28,28 744x45.4"),
        "{layout}"
    );
    let label = app.find(role(Role::Text).name("Disk")).bounds;
    assert!((label.size.height - 14.4).abs() < 0.01);
    assert_eq!(label.origin.y, 28.0 + 24.0 + 7.0);
}

#[test]
fn each_step_owns_an_equal_column_with_its_dot_centered_in_it() {
    let mut app = TestApp::new(rail_only(Theme::dark(), STEPS.to_vec(), 0));
    let layout = app.layout_snapshot();
    for (index, fraction) in [0.125, 0.375, 0.625, 0.875].into_iter().enumerate() {
        let dot_x: f32 = 400.0 * fraction - 12.0;
        let marker = if index == 0 {
            "kit_wizard_rail > stack[1] > stack @".to_owned()
        } else {
            format!("kit_wizard_rail > stack[{}] > stack @", index + 1)
        };
        let line = layout
            .lines()
            .find(|line| line.contains(&marker) && line.contains("24x24"))
            .unwrap_or_else(|| panic!("no dot for step {index}:\n{layout}"));
        let x: f32 = line
            .split('@')
            .nth(1)
            .and_then(|rest| rest.split(',').next())
            .and_then(|x| x.parse().ok())
            .unwrap_or_default();
        assert!((x - dot_x).abs() < 0.01, "step {index}: {x} vs {dot_x}");
    }
}

#[test]
fn done_and_current_dots_take_the_accent_and_pending_ones_a_faint_track() {
    let theme = Theme::dark();
    let app = TestApp::new(rail_only(theme, STEPS.to_vec(), 2));
    let frame = app.frame_snapshot();
    let accent = format!("fill 0,0 24x24 r12 {}", hex(theme.accent.end));
    let pending = format!(
        "fill 0,0 24x24 r12 {}",
        hex(theme.control.track.with_alpha_factor(0.55))
    );
    assert_eq!(frame.matches(&accent).count(), 3, "{frame}");
    assert_eq!(frame.matches(&pending).count(), 1, "{frame}");
}

#[test]
fn completed_steps_show_a_check_and_only_the_current_one_a_ring() {
    let app = TestApp::new(rail_only(Theme::dark(), STEPS.to_vec(), 2));
    let frame = app.frame_snapshot();
    assert_eq!(checks(&frame), 2, "{frame}");
    assert_eq!(frame.matches(RING).count(), 1, "{frame}");
    assert!(frame.contains(&format!("{RING} {}", hex(Theme::dark().accent.end))));
}

#[test]
fn the_ring_sits_two_and_a_half_pixels_outside_the_dot() {
    let mut app = TestApp::new(rail_only(Theme::dark(), STEPS.to_vec(), 0));
    assert!(
        app.layout_snapshot()
            .contains("33x33 absolute rounded-16.5")
    );
    assert!(app.frame_snapshot().contains("layer at -4.5,-4.5"));
}

#[test]
fn connectors_are_two_pixel_tracks_that_fill_in_behind_the_current_step() {
    let theme = Theme::dark();
    let app = TestApp::new(rail_only(theme, STEPS.to_vec(), 2));
    let frame = app.frame_snapshot();
    assert!(
        frame.contains(&format!("fill 62,11 76x2 {}", hex(theme.accent.end))),
        "{frame}"
    );
    assert!(
        frame.contains(&format!("fill 162,11 76x2 {}", hex(theme.accent.end))),
        "{frame}"
    );
    assert!(
        frame.contains(&format!(
            "fill 262,11 76x2 {}",
            hex(theme.control.track.with_alpha_factor(0.55))
        )),
        "{frame}"
    );
}

#[test]
fn advancing_moves_the_ring_fills_the_track_and_checks_the_step_left_behind() {
    let mut app = TestApp::new(flow(Theme::dark(), 0));
    let before = app.frame_snapshot();
    assert_eq!(checks(&before), 0);
    app.click(role(Role::Button).name("Continue"));
    app.click(role(Role::Button).name("Continue"));
    let after = app.frame_snapshot();
    assert_eq!(checks(&after), 2, "{after}");
    assert_eq!(after.matches(RING).count(), 1);
    assert_ne!(before, after);
    assert!(!app.is_animating());
}

#[test]
fn a_current_past_the_end_completes_every_step() {
    let app = TestApp::new(rail_only(Theme::dark(), STEPS.to_vec(), STEPS.len()));
    let frame = app.frame_snapshot();
    assert_eq!(checks(&frame), 4, "{frame}");
    assert_eq!(frame.matches(RING).count(), 0);
}

#[test]
fn an_empty_rail_paints_nothing_and_does_not_panic() {
    let mut app = TestApp::new(rail_only(Theme::dark(), Vec::new(), 0));
    assert!(app.layout_snapshot().contains("kit_wizard_rail @0,0 400x0"));
    assert_eq!(checks(&app.frame_snapshot()), 0);
}

#[test]
fn the_rail_announces_where_the_user_is() {
    let mut app = TestApp::new(flow(Theme::dark(), 1));
    assert_eq!(
        app.find(role(Role::ProgressBar)).value.as_deref(),
        Some("Step 2 of 4: Disk")
    );
    app.click(role(Role::Button).name("Continue"));
    assert_eq!(
        app.find(role(Role::ProgressBar)).value.as_deref(),
        Some("Step 3 of 4: Review")
    );
}

#[test]
fn the_rail_is_inert() {
    let mut app = TestApp::new(flow(Theme::dark(), 1));
    let disk = app.find(role(Role::Text).name("Review")).bounds.center();
    app.pointer_down(disk.x, disk.y);
    app.pointer_up(disk.x, disk.y);
    assert_eq!(
        app.find(role(Role::ProgressBar)).value.as_deref(),
        Some("Step 2 of 4: Disk")
    );
}

#[test]
fn the_frame_pads_twenty_eight_pixels_and_pins_the_footer_bottom_right() {
    let mut app = TestApp::new(flow(Theme::dark(), 1));
    let back = app.find(role(Role::Button).name("Back")).bounds;
    let forward = app.find(role(Role::Button).name("Continue")).bounds;
    assert_eq!(back.origin.x, 28.0);
    assert_close(forward.origin.x + forward.size.width, 800.0 - 28.0);
    assert_close(forward.origin.y + forward.size.height, 600.0 - 28.0);
    assert_eq!(app.find(role(Role::ProgressBar)).bounds.origin.y, 28.0);
}

#[test]
fn the_page_absorbs_a_resize_and_the_footer_stays_put() {
    let mut app = TestApp::new(flow(Theme::dark(), 1));
    app.resize(1000, 400);
    let forward = app.find(role(Role::Button).name("Continue")).bounds;
    assert_close(forward.origin.x + forward.size.width, 1000.0 - 28.0);
    assert_close(forward.origin.y + forward.size.height, 400.0 - 28.0);
}

#[test]
fn a_hairline_separates_the_page_from_the_footer_with_sixteen_pixels_each_side() {
    let mut app = TestApp::new(flow(Theme::dark(), 1));
    let separator = app.find(role(Role::Separator)).bounds;
    let forward = app.find(role(Role::Button).name("Continue")).bounds;
    assert_eq!(separator.size.height, 1.0);
    assert_eq!(separator.origin.x, 28.0);
    assert_eq!(separator.size.width, 744.0);
    assert_eq!(forward.origin.y - (separator.origin.y + 1.0), 16.0);
    assert!(app.frame_snapshot().contains(&format!(
        "fill 0,0 744x1 {}",
        hex(Theme::dark().surface.border)
    )));
}

#[test]
fn the_page_stacks_title_subtitle_and_content_with_the_documented_gaps() {
    let mut app = TestApp::new(flow(Theme::dark(), 1));
    let title = app.find(text_query("How much room?")).bounds;
    let subtitle = app.find(text_query("Windows keeps everything.")).bounds;
    let content = app.find(text_query("controls")).bounds;
    assert!((title.size.height - 21.0 * 1.2).abs() < 0.01);
    assert!((subtitle.origin.y - (title.origin.y + title.size.height + 6.0)).abs() < 0.01);
    assert!((content.origin.y - (subtitle.origin.y + subtitle.size.height + 20.0)).abs() < 0.01);
    assert_eq!(title.origin.x, 28.0);
    assert_eq!(content.size.width, 744.0);
}

#[test]
fn a_page_without_a_subtitle_goes_straight_from_title_to_content() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        wizard(
            STEPS,
            Prop::Static(0),
            wizard_page("Title", None::<&str>, text("body")),
            text("footer"),
        )
    }));
    let title = app.find(text_query("Title")).bounds;
    let body = app.find(text_query("body")).bounds;
    assert!((body.origin.y - (title.origin.y + title.size.height + 20.0)).abs() < 0.01);
}

#[test]
fn a_scrolled_step_never_pushes_the_footer_off_the_window() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        wizard(
            STEPS,
            Prop::Static(0),
            wizard_page(
                "Disks",
                None::<&str>,
                wizard_scroll(vstack(
                    (0..60)
                        .map(|index| text(format!("disk {index}")))
                        .collect::<Vec<_>>(),
                )),
            ),
            wizard_footer(button("Back"), button("Continue")),
        )
    }));
    let forward = app.find(role(Role::Button).name("Continue")).bounds;
    assert_close(forward.origin.y + forward.size.height, 572.0);
    let scroll = app.find(role(Role::ScrollView)).bounds;
    assert!(scroll.size.height < 60.0 * 16.8);
    assert!(scroll.origin.y + scroll.size.height <= 572.0);
    app.scroll(role(Role::ScrollView), 200.0);
    assert!(app.exists(role(Role::ScrollView)));
}

#[test]
fn the_frame_and_labels_follow_the_palette() {
    let dark = Theme::dark();
    let light = Theme::light();
    let dark_app = TestApp::new(flow(dark, 1));
    let light_app = TestApp::new(flow(light, 1));
    assert!(
        dark_app
            .frame_snapshot()
            .contains(&format!("fill 0,0 800x600 {}", hex(dark.surface.bg)))
    );
    assert!(
        light_app
            .frame_snapshot()
            .contains(&format!("fill 0,0 800x600 {}", hex(light.surface.bg)))
    );
    assert!(light_app.frame_snapshot().contains(&hex(light.accent.end)));
}

#[test]
fn switching_themes_repaints_the_rail() {
    let mut app = TestApp::new(component(|cx| {
        let theme = provide_theme(cx, Theme::dark());
        vstack((
            button("Switch").on_click(move |cx, _| theme.set(cx, Theme::light())),
            wizard_rail(STEPS, Prop::Static(1)),
        ))
        .w(400.0)
    }));
    let dark_pending = hex(Theme::dark().control.track.with_alpha_factor(0.55));
    assert!(app.frame_snapshot().contains(&dark_pending));
    app.click(role(Role::Button).name("Switch"));
    app.settle();
    let frame = app.frame_snapshot();
    assert!(!frame.contains(&dark_pending), "{frame}");
    assert!(
        frame.contains(&hex(Theme::light().control.track.with_alpha_factor(0.55))),
        "{frame}"
    );
}

#[test]
fn every_stage_of_the_wizard_paints_the_palette() {
    let app = TestApp::new(flow(Theme::dark(), 2));
    insta::assert_snapshot!(app.frame_snapshot());
}
