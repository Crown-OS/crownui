use crowntest::{Role, TestApp, role, text as text_query};
use view::{Listeners, Styled, component};
use widgets::{
    hstack,
    kit::{
        ButtonOptions, ButtonVariant, HeroStep, Theme, button, hero_pane, hero_wizard,
        provide_theme,
    },
    text,
};

const WINDOW_WIDTH: f32 = 1000.0;
const WINDOW_HEIGHT: f32 = 700.0;
const CARD_LEFT: f32 = 57.0;
const CARD_TOP: f32 = 63.0;

fn wizard_app(theme: Theme, step: HeroStep, content: &'static str) -> TestApp {
    let mut app = TestApp::with_size(
        component(move |cx| {
            provide_theme(cx, theme);
            hero_wizard(
                hero_pane(text("Brand")),
                step,
                text(content),
                button("Next step")
                    .variant(ButtonVariant::Primary)
                    .on_click(|_, _| {}),
            )
        }),
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
    );
    app.settle();
    app
}

fn disk_step() -> HeroStep {
    HeroStep::new("Please select the disk to install to")
        .subtitle("After selecting the disk, click Next")
        .steps(4, 1_usize)
}

fn disk_app() -> TestApp {
    wizard_app(Theme::dark(), disk_step(), "Disk A")
}

#[test]
fn the_card_is_886_by_574_and_centred_in_the_window() {
    let mut app = disk_app();
    let layout = app.layout_snapshot();
    assert!(
        layout.contains(&format!("kit_pane @{CARD_LEFT},{CARD_TOP} 886x574")),
        "{layout}"
    );
}

#[test]
fn the_card_stays_centred_as_the_window_resizes() {
    let mut app = disk_app();
    app.resize(1200, 800);
    let layout = app.layout_snapshot();
    assert!(layout.contains("kit_pane @157,113 886x574"), "{layout}");
}

#[test]
fn the_hero_panel_is_366_wide_and_as_tall_as_the_card() {
    let mut app = disk_app();
    let layout = app.layout_snapshot();
    assert!(
        layout.contains(&format!("kit_pane[1] @{CARD_LEFT},{CARD_TOP} 366x574")),
        "{layout}"
    );
}

#[test]
fn the_hero_content_is_centred_in_the_panel() {
    let mut app = disk_app();
    let brand = app.find(text_query("Brand")).bounds;
    let centre_x = brand.origin.x + brand.size.width / 2.0;
    let centre_y = brand.origin.y + brand.size.height / 2.0;
    assert!((centre_x - (CARD_LEFT + 183.0)).abs() < 0.01, "{brand:?}");
    assert!((centre_y - (CARD_TOP + 287.0)).abs() < 0.01, "{brand:?}");
}

#[test]
fn the_title_starts_inside_the_34_pixel_column_padding_at_26_pixels() {
    let mut app = disk_app();
    let title = app.find(text_query("Please select the disk to install to"));
    assert_eq!(title.bounds.origin.x, CARD_LEFT + 366.0 + 34.0);
    assert_eq!(title.bounds.origin.y, CARD_TOP + 34.0);
    assert!((title.bounds.size.height - 2.0 * 26.0 * 1.2).abs() < 0.01);
}

#[test]
fn the_title_wraps_at_the_452_pixel_measure() {
    let mut app = wizard_app(
        Theme::dark(),
        HeroStep::new(
            "Please select the disk to install to, then confirm that everything on it can be erased",
        ),
        "Disk A",
    );
    let title = app
        .find(text_query(
            "Please select the disk to install to, then confirm that everything on it can be erased",
        ))
        .bounds;
    assert!(title.size.width <= 452.01, "{title:?}");
    assert!(title.size.height > 2.0 * 26.0 * 1.2, "{title:?}");
}

#[test]
fn the_subtitle_sits_eight_pixels_under_the_title_at_15_pixels() {
    let mut app = disk_app();
    let title = app
        .find(text_query("Please select the disk to install to"))
        .bounds;
    let subtitle = app
        .find(text_query("After selecting the disk, click Next"))
        .bounds;
    assert_eq!(subtitle.origin.y, title.max_y() + 8.0);
    assert!(
        (subtitle.size.height - 15.0 * 1.2).abs() < 0.01,
        "{subtitle:?}"
    );
}

#[test]
fn the_step_bar_is_452_wide_and_26_pixels_under_the_subtitle() {
    let mut app = disk_app();
    let subtitle = app
        .find(text_query("After selecting the disk, click Next"))
        .bounds;
    let bar = app.find(role(Role::ProgressBar)).bounds;
    assert_eq!(bar.size.width, 452.0);
    assert_eq!(bar.size.height, 5.0);
    assert_eq!(bar.origin.x, CARD_LEFT + 366.0 + 34.0);
    assert_eq!(bar.origin.y, subtitle.max_y() + 8.0 + 18.0);
}

#[test]
fn the_content_sits_26_pixels_under_the_step_bar() {
    let mut app = disk_app();
    let bar = app.find(role(Role::ProgressBar)).bounds;
    let content = app.find(text_query("Disk A")).bounds;
    assert_eq!(content.origin.y, bar.max_y() + 26.0);
    assert_eq!(content.origin.x, bar.origin.x);
}

#[test]
fn without_a_subtitle_the_bar_follows_the_title_directly() {
    let mut app = wizard_app(
        Theme::dark(),
        HeroStep::new("Almost there").steps(3, 0_usize),
        "Disk A",
    );
    let title = app.find(text_query("Almost there")).bounds;
    let bar = app.find(role(Role::ProgressBar)).bounds;
    assert_eq!(bar.origin.y, title.max_y() + 8.0 + 18.0);
}

#[test]
fn a_step_without_steps_has_no_bar() {
    let mut app = wizard_app(Theme::dark(), HeroStep::new("All done"), "Disk A");
    assert!(!app.exists(role(Role::ProgressBar)));
}

#[test]
fn the_step_bar_follows_a_signal() {
    let mut app = TestApp::with_size(
        component(|cx| {
            provide_theme(cx, Theme::dark());
            let step = cx.signal(0_usize);
            hero_wizard(
                hero_pane(text("Brand")),
                HeroStep::new("Title").steps(4, step),
                (),
                button("Next step").on_click(move |cx, _| step.update(cx, |step| *step += 1)),
            )
        }),
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
    );
    app.settle();
    assert_eq!(
        app.find(role(Role::ProgressBar)).value.as_deref(),
        Some("step 1 of 4")
    );
    app.click(role(Role::Button).name("Next step"));
    app.settle();
    assert_eq!(
        app.find(role(Role::ProgressBar)).value.as_deref(),
        Some("step 2 of 4")
    );
}

#[test]
fn the_action_is_pinned_to_the_foot_of_the_column_whatever_the_content() {
    let mut short = wizard_app(Theme::dark(), HeroStep::new("Title"), "Disk A");
    let mut long = disk_app();
    for app in [&mut short, &mut long] {
        let action = app.find(role(Role::Button)).bounds;
        assert_eq!(action.max_y(), CARD_TOP + 574.0 - 34.0, "{action:?}");
    }
}

#[test]
fn the_action_stretches_across_the_452_pixel_column() {
    let mut app = disk_app();
    let action = app.find(role(Role::Button)).bounds;
    assert_eq!(action.size.width, 452.0);
    assert_eq!(action.origin.x, CARD_LEFT + 366.0 + 34.0);
}

#[test]
fn a_row_of_buttons_as_the_action_is_stretched_across_the_column() {
    let mut app = TestApp::with_size(
        component(|cx| {
            provide_theme(cx, Theme::dark());
            hero_wizard(
                hero_pane(text("Brand")),
                HeroStep::new("Title"),
                (),
                hstack((
                    button("Back").grow(1.0),
                    button("Restart").variant(ButtonVariant::Primary).grow(1.0),
                ))
                .gap(10.0),
            )
        }),
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
    );
    app.settle();
    let back = app.find(role(Role::Button).name("Back")).bounds;
    let restart = app.find(role(Role::Button).name("Restart")).bounds;
    assert_eq!(back.origin.x, CARD_LEFT + 366.0 + 34.0);
    assert!((restart.max_x() - (CARD_LEFT + 886.0 - 34.0)).abs() < 0.01);
    assert_eq!(restart.origin.x - back.max_x(), 10.0);
}

#[test]
fn the_page_is_the_background_and_the_card_the_raised_surface_at_radius_22() {
    let app = disk_app();
    let frame = app.frame_snapshot();
    assert!(frame.contains("#0f0f10cc"), "{frame}");
    assert!(frame.contains("r22 #171718ff"), "{frame}");
}

#[test]
fn the_card_casts_the_soft_shadow_at_its_radius() {
    let app = disk_app();
    let frame = app.frame_snapshot();
    let shadow = frame
        .lines()
        .find(|line| line.contains("shadow"))
        .unwrap_or_default();
    assert!(
        shadow.contains("r22") && shadow.contains("blur20"),
        "{frame}"
    );
}

#[test]
fn the_hero_panel_paints_the_accent_gradient_square_at_the_seam() {
    let app = disk_app();
    let frame = app.frame_snapshot();
    assert!(
        frame.contains("gradient(#8f6dfbff@0,#6d48e8ff@1)"),
        "{frame}"
    );
}

#[test]
fn switching_the_theme_repaints_the_page_and_card() {
    let mut app = TestApp::with_size(
        component(|cx| {
            let theme = provide_theme(cx, Theme::dark());
            hero_wizard(
                hero_pane(text("Brand")),
                HeroStep::new("Title"),
                (),
                button("Light").on_click(move |cx, _| theme.set(cx, Theme::light())),
            )
        }),
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
    );
    app.settle();
    let dark = app.frame_snapshot();
    app.click(role(Role::Button).name("Light"));
    app.advance_ms(16);
    assert!(app.is_animating());
    app.settle();
    assert_ne!(dark, app.frame_snapshot());
    assert!(!app.is_animating());
}

#[test]
fn mounting_does_not_animate() {
    let app = disk_app();
    assert!(!app.is_animating());
}

#[test]
fn a_composed_card_matches_its_snapshot() {
    let mut app = disk_app();
    insta::assert_snapshot!(app.snapshot());
}
