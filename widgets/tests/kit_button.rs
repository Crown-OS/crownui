use crowntest::{Role, TestApp, role, text as text_query};
use view::{Listeners, Semantic, Styled, bind, component};
use widgets::{
    kit::{
        ButtonOptions, ButtonSize, ButtonVariant, Theme, button, icon_button, icons, provide_theme,
    },
    text, vstack,
};

fn contains_after_settling(app: &mut TestApp, needle: &str) -> bool {
    app.settle();
    app.frame_snapshot().contains(needle)
}

#[test]
fn clicking_fires_on_click_unless_the_button_is_disabled() {
    let mut app = TestApp::new(component(|cx| {
        let presses = cx.signal(0);
        let blocked = cx.signal(true);
        vstack((
            button("Save").on_click(move |cx, _| presses.update(cx, |count| *count += 1)),
            button("Blocked")
                .disabled(blocked)
                .on_click(move |cx, _| presses.update(cx, |count| *count += 100)),
            text(bind(move |runtime| {
                format!("Pressed {}", presses.get(runtime))
            })),
        ))
    }));
    app.click(role(Role::Button).name("Save"));
    assert_eq!(app.text_of(text_query("Pressed")), "Pressed 1");
    assert!(app.find(role(Role::Button).name("Blocked")).states.disabled);
}

#[test]
fn a_hovered_button_washes_at_seven_tenths_and_a_pressed_one_at_full_strength() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        vstack(button("Save")).p(24.0)
    }));
    assert!(!app.frame_snapshot().contains("opacity"));

    app.hover(role(Role::Button).name("Save"));
    assert!(
        contains_after_settling(&mut app, "layer opacity0.7"),
        "{}",
        app.frame_snapshot()
    );

    let bounds = app.find(role(Role::Button).name("Save")).bounds;
    app.pointer_down(bounds.center().x, bounds.center().y);
    app.settle();
    let pressed = app.frame_snapshot();
    assert!(
        pressed.contains("layer\n      fill 0,0 73.6x34 r17 #ffffff10"),
        "{pressed}"
    );
    assert!(!pressed.contains("opacity"), "{pressed}");
}

#[test]
fn the_wash_eases_in_instead_of_snapping() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        vstack(button("Save")).p(24.0)
    }));
    app.hover(role(Role::Button).name("Save"));
    app.advance_ms(16);
    assert!(app.is_animating());
    app.settle();
    assert!(!app.is_animating());
}

#[test]
fn keyboard_focus_draws_no_halo_around_the_pill() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        vstack(button("Save").variant(ButtonVariant::Primary)).p(24.0)
    }));
    assert!(!app.frame_snapshot().contains("stroke3"));
    app.focus(role(Role::Button).name("Save"));
    app.settle();
    let frame = app.frame_snapshot();
    assert!(!frame.contains("stroke3"), "{frame}");
}

#[test]
fn every_variant_paints_its_palette_fill() {
    let theme = Theme::dark();
    let app = TestApp::new(component(move |cx| {
        provide_theme(cx, theme);
        vstack((
            button("Primary").variant(ButtonVariant::Primary),
            button("Secondary"),
            button("Ghost").variant(ButtonVariant::Ghost),
            button("Destructive").variant(ButtonVariant::Destructive),
            button("Tinted").variant(ButtonVariant::Tinted),
        ))
        .p(24.0)
        .gap(8.0)
    }));
    insta::assert_snapshot!(app.frame_snapshot());
}

#[test]
fn switching_themes_repaints_the_fill() {
    let mut app = TestApp::new(component(|cx| {
        let theme = provide_theme(cx, Theme::light());
        vstack((
            button("Primary").variant(ButtonVariant::Primary),
            button("Switch").on_click(move |cx, _| theme.set(cx, Theme::dark())),
        ))
        .p(24.0)
    }));
    let dark_chip = "gradient(#1c1c1cff@0,#1c1c1cff@1)";
    assert!(!app.frame_snapshot().contains(dark_chip));
    app.click(role(Role::Button).name("Switch"));
    assert!(
        contains_after_settling(&mut app, dark_chip),
        "{}",
        app.frame_snapshot()
    );
}

#[test]
fn an_icon_button_is_a_square_named_by_its_label() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        vstack(icon_button(icons::CHECK, "Confirm").variant(ButtonVariant::Ghost))
            .p(24.0)
            .items_start()
    }));
    let bounds = app.find(role(Role::Button).name("Confirm")).bounds;
    assert_eq!((bounds.size.width, bounds.size.height), (28.0, 28.0));
}

#[test]
fn sizes_set_the_height_and_stretch_fills_the_column() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        vstack((
            button("Small").control_size(ButtonSize::Small),
            button("Medium"),
            button("Wide").stretch(),
        ))
        .p(24.0)
        .w(300.0)
    }));
    assert_eq!(
        app.find(role(Role::Button).name("Small"))
            .bounds
            .size
            .height,
        28.0
    );
    assert_eq!(
        app.find(role(Role::Button).name("Medium"))
            .bounds
            .size
            .height,
        34.0
    );
    assert_eq!(
        app.find(role(Role::Button).name("Wide")).bounds.size.width,
        252.0
    );
}
