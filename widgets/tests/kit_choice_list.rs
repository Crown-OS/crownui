use crowntest::{Role, TestApp, role, text as text_query};
use view::{Listeners, Semantic, Styled, bind, component};
use widgets::{
    kit::{Choice, Theme, button, choice_list, icons, provide_theme},
    text, vstack,
};

fn disks() -> [Choice; 3] {
    [
        Choice::new("/dev/nvme0n1", "476 GB · Windows 11 · 94 GiB free").icon(icons::HARD_DRIVE),
        Choice::new("/dev/sdb", "no free space")
            .icon(icons::HARD_DRIVE)
            .disabled(),
        Choice::new("/dev/sdc", "1 TB"),
    ]
}

fn disk_picker(initial: Option<usize>) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, Theme::dark());
        let selected = cx.signal(initial);
        vstack((
            choice_list(disks(), selected).label("Disk"),
            text(bind(move |runtime| {
                format!("Picked {:?}", selected.get(runtime))
            })),
        ))
        .p(24.0)
        .w(400.0)
    }))
}

fn row(name: &str) -> crowntest::Query {
    role(Role::RadioButton).name(name)
}

fn picked(app: &mut TestApp) -> String {
    app.text_of(text_query("Picked"))
}

fn settled_frame(app: &mut TestApp) -> String {
    app.settle();
    app.frame_snapshot()
}

#[test]
fn rows_are_seventy_two_tall_eight_apart_and_fill_the_width_offered() {
    let mut app = disk_picker(Some(0));
    let list = app.find(role(Role::Group).name("Disk")).bounds;
    assert_eq!(
        (list.size.width, list.size.height),
        (352.0, 3.0 * 72.0 + 2.0 * 8.0)
    );
    for (index, name) in ["/dev/nvme0n1", "/dev/sdb", "/dev/sdc"].iter().enumerate() {
        let bounds = app.find(row(name)).bounds;
        assert_eq!((bounds.size.width, bounds.size.height), (352.0, 72.0));
        assert_eq!(bounds.origin.y, list.origin.y + 80.0 * index as f32);
    }
}

#[test]
fn the_tile_the_text_and_the_radio_sit_at_the_original_offsets() {
    let mut app = disk_picker(Some(0));
    let first = app.find(row("/dev/nvme0n1")).bounds;
    let title = app.find(role(Role::Text).name("/dev/nvme0n1")).bounds;
    assert_eq!(title.origin.x, first.origin.x + 14.0 + 44.0 + 14.0);
    let plain = app.find(row("/dev/sdc")).bounds;
    let plain_title = app.find(role(Role::Text).name("/dev/sdc")).bounds;
    assert_eq!(plain_title.origin.x, plain.origin.x + 14.0);
    let frame = app.frame_snapshot();
    assert!(frame.contains("layer at 14,14"), "{frame}");
    assert!(frame.contains("fill 0,0 44x44 r11 #6d48e829"), "{frame}");
    assert!(frame.contains("layer at 320,27"), "{frame}");
}

#[test]
fn the_whole_row_is_the_target_and_the_gap_between_rows_is_not() {
    let mut app = disk_picker(None);
    let third = app.find(row("/dev/sdc")).bounds;
    app.pointer_down(
        third.origin.x + third.size.width - 3.0,
        third.origin.y + 3.0,
    );
    app.pointer_up(
        third.origin.x + third.size.width - 3.0,
        third.origin.y + 3.0,
    );
    assert_eq!(picked(&mut app), "Picked Some(2)");
    let first = app.find(row("/dev/nvme0n1")).bounds;
    let gap_y = first.origin.y + first.size.height + 4.0;
    app.pointer_down(first.origin.x + 20.0, gap_y);
    app.pointer_up(first.origin.x + 20.0, gap_y);
    assert_eq!(picked(&mut app), "Picked Some(2)");
}

#[test]
fn a_disabled_row_cannot_be_picked_hovered_or_marked_and_is_drawn_faded() {
    let mut app = disk_picker(Some(0));
    assert!(app.find(row("/dev/sdb")).states.disabled);
    let bounds = app.find(row("/dev/sdb")).bounds;
    app.pointer_down(bounds.center().x, bounds.center().y);
    app.pointer_up(bounds.center().x, bounds.center().y);
    assert_eq!(picked(&mut app), "Picked Some(0)");
    app.pointer_move(bounds.center().x, bounds.center().y);
    let frame = settled_frame(&mut app);
    assert!(!frame.contains("#ffffff10"), "{frame}");
    assert!(frame.contains("fill 0,0 352x72 r14 #17171873"), "{frame}");
    assert!(
        frame.contains("gradient(#27272a73@0,#27272a73@1)"),
        "{frame}"
    );
    assert_eq!(frame.matches("fill 0,0 18x18 r9").count(), 3, "{frame}");
}

#[test]
fn a_disabled_list_fades_every_row_and_ignores_input() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let selected = cx.signal(Some(0usize));
        vstack((
            choice_list(disks(), selected).disabled(true),
            text(bind(move |runtime| {
                format!("Picked {:?}", selected.get(runtime))
            })),
        ))
        .p(24.0)
    }));
    let bounds = app.find(row("/dev/sdc")).bounds;
    app.pointer_down(bounds.center().x, bounds.center().y);
    app.pointer_up(bounds.center().x, bounds.center().y);
    assert_eq!(picked(&mut app), "Picked Some(0)");
    let frame = settled_frame(&mut app);
    assert_eq!(
        frame.matches("fill 0,0 752x72 r14 #17171873").count(),
        3,
        "{frame}"
    );
    assert!(!frame.contains("#ffffff10"), "{frame}");
}

#[test]
fn hovering_washes_a_row_on_the_snappy_spring() {
    let mut app = disk_picker(Some(0));
    assert!(!app.frame_snapshot().contains("#ffffff10"));
    app.hover(row("/dev/sdc"));
    app.advance_ms(16);
    assert!(app.is_animating());
    assert!(app.frame_snapshot().contains("opacity"));
    let frame = settled_frame(&mut app);
    assert!(frame.contains("fill 0,0 352x72 r14 #ffffff10"), "{frame}");
    assert!(!app.is_animating());
}

#[test]
fn the_selected_row_tints_borders_and_marks_itself_with_the_accent() {
    let mut app = disk_picker(Some(0));
    let frame = settled_frame(&mut app);
    assert!(frame.contains("layer opacity0.07"), "{frame}");
    assert!(
        frame.contains("stroke1 0.5,0.5 351x71 r13.5 gradient(#8f6dfbff@0,#6d48e8ff@1)"),
        "{frame}"
    );
    assert!(
        frame.contains("stroke1.8 path[2] 2,12 20x0 #6d48e8ff"),
        "{frame}"
    );
    assert!(
        frame.contains("transform [0.5, 0.0, 0.0, 0.5, 5.0, 5.0]"),
        "{frame}"
    );
}

#[test]
fn choosing_another_row_cross_fades_both_rows_instead_of_snapping() {
    let mut app = disk_picker(Some(0));
    assert!(!app.is_animating());
    app.click(row("/dev/sdc"));
    app.advance_ms(16);
    assert!(app.is_animating());
    let midway = app.frame_snapshot();
    assert!(
        midway.contains("opacity0.0") && midway.contains("opacity"),
        "{midway}"
    );
    let settled = settled_frame(&mut app);
    assert!(!app.is_animating());
    assert_eq!(settled.matches("layer opacity0.07").count(), 1, "{settled}");
    assert_eq!(app.find(row("/dev/sdc")).states.checked, Some(true));
    assert_eq!(app.find(row("/dev/nvme0n1")).states.checked, Some(false));
}

#[test]
fn a_selection_past_the_end_of_the_list_marks_nothing() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let selected = cx.signal(Some(4usize));
        vstack(choice_list([Choice::new("only", "")], selected)).p(24.0)
    }));
    assert_eq!(app.find(row("only")).states.checked, Some(false));
    assert_eq!(app.find(role(Role::Group)).value.as_deref(), Some(""));
}

#[test]
fn arrows_skip_unpickable_rows_and_wrap_and_enter_does_nothing() {
    let mut app = disk_picker(Some(0));
    app.focus(role(Role::Group).name("Disk"));
    app.press_key("ArrowDown");
    assert_eq!(picked(&mut app), "Picked Some(2)");
    app.press_key("ArrowDown");
    assert_eq!(picked(&mut app), "Picked Some(0)");
    app.press_key("ArrowUp");
    assert_eq!(picked(&mut app), "Picked Some(2)");
    app.press_key("Enter");
    assert_eq!(picked(&mut app), "Picked Some(2)");
}

#[test]
fn keyboard_focus_draws_no_ring_around_the_current_row() {
    let mut app = disk_picker(Some(2));
    app.focus(role(Role::Group).name("Disk"));
    let frame = settled_frame(&mut app);
    assert!(!frame.contains("stroke3"), "{frame}");
}

#[test]
fn assistive_technology_reads_titles_details_and_the_picked_value() {
    let mut app = disk_picker(Some(0));
    let group = app.find(role(Role::Group).name("Disk"));
    assert_eq!(group.value.as_deref(), Some("/dev/nvme0n1"));
    let node = app.find(row("/dev/nvme0n1"));
    assert_eq!(
        node.description.as_deref(),
        Some("476 GB · Windows 11 · 94 GiB free")
    );
    assert_eq!(node.states.checked, Some(true));
    assert!(!node.states.focusable);
}

#[test]
fn the_palette_follows_a_theme_switch() {
    let mut app = TestApp::new(component(|cx| {
        let theme = provide_theme(cx, Theme::light());
        let selected = cx.signal(Some(0usize));
        vstack((
            choice_list(disks(), selected),
            button("Dark").on_click(move |cx, _| theme.set(cx, Theme::dark())),
        ))
        .p(24.0)
    }));
    assert!(
        app.frame_snapshot()
            .contains("fill 0,0 752x72 r14 #ffffffff")
    );
    app.click(role(Role::Button).name("Dark"));
    let dark = settled_frame(&mut app);
    assert!(dark.contains("fill 0,0 752x72 r14 #171718ff"), "{dark}");
}

#[test]
fn a_composed_frame_matches_its_snapshot() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::light());
        let selected = cx.signal(Some(0usize));
        vstack(choice_list(disks(), selected)).p(24.0).w(420.0)
    }));
    insta::assert_snapshot!(settled_frame(&mut app));
}
