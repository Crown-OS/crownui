use crowntest::{Role, TestApp, role, text as text_query};
use reactive::Signal;
use view::{Listeners, Semantic, Styled, View, bind, component};
use widgets::{
    kit::{
        SearchOptions, Theme, button, icons, provide_theme, sidebar, sidebar_brand, sidebar_column,
        sidebar_group, sidebar_item, sidebar_search, sidebar_section, sidebar_separator,
        sidebar_subitem, sidebar_with_header,
    },
    spacer, text,
};

const INBOX: usize = 0;
const TASKS: usize = 1;
const LOCAL: usize = 2;

fn sidebar_demo(theme: Theme) -> impl View {
    component(move |cx| {
        provide_theme(cx, theme);
        let page = cx.signal(INBOX);
        let open = cx.signal(true);
        let go = move |target: usize| move |cx: &mut view::Cx, _: &mut _| page.set(cx, target);
        sidebar(
            (
                sidebar_brand("Acme", icons::HOUSE),
                sidebar_section(
                    "Personal",
                    (
                        sidebar_item(
                            "Inbox",
                            icons::INBOX,
                            bind(move |runtime| page.get(runtime) == INBOX),
                        )
                        .on_click(go(INBOX)),
                        sidebar_item(
                            "Tasks",
                            icons::CIRCLE_DOT,
                            bind(move |runtime| page.get(runtime) == TASKS),
                        )
                        .on_click(go(TASKS)),
                        sidebar_item("Archive", icons::INBOX, false).disabled(true),
                    ),
                ),
                sidebar_separator(),
                sidebar_group(
                    "Finance",
                    icons::HOUSE,
                    open,
                    sidebar_column((
                        sidebar_subitem("Local", bind(move |runtime| page.get(runtime) == LOCAL))
                            .on_click(go(LOCAL)),
                        sidebar_subitem("Other", false),
                    )),
                ),
                sidebar_item("Settings", icons::SETTINGS, false),
            ),
            text("content"),
        )
    })
}

fn bounds(app: &mut TestApp, role_kind: Role, name: &str) -> geometry::Rect {
    app.find(role(role_kind).name(name)).bounds
}

fn item_bounds(app: &mut TestApp, name: &str) -> geometry::Rect {
    bounds(app, Role::MenuItem, name)
}

fn fill_row(color: &str) -> String {
    format!("fill 0,0 235x35 r16 {color}")
}

const PILL_FILL: &str = "#232326ff";
const PILL_BORDER: &str = "stroke1 0.5,0.5 235x35 r16 #ffffff1c";
const HOVER_FILL: &str = "#ffffff0e";

fn clip_height(frame: &str) -> f32 {
    let line = frame
        .lines()
        .find(|line| line.contains("layer clip 0,0 236x"))
        .unwrap_or_else(|| panic!("no collapse clip in:\n{frame}"));
    let height = line.rsplit('x').next().unwrap_or_default();
    height.parse().unwrap_or_else(|_| panic!("bad clip {line}"))
}

#[test]
fn rows_are_thirty_six_tall_and_fill_the_menu_column() {
    let mut app = TestApp::new(sidebar_demo(Theme::dark()));
    let inbox = item_bounds(&mut app, "Inbox");
    assert_eq!((inbox.size.width, inbox.size.height), (236.0, 36.0));
    assert_eq!(inbox.origin.x, 18.0);
    assert_eq!(item_bounds(&mut app, "Local").size.height, 36.0);
}

#[test]
fn the_brand_row_is_forty_four_tall_with_a_bigger_icon_and_label() {
    let mut app = TestApp::new(sidebar_demo(Theme::dark()));
    assert!(
        app.layout_snapshot()
            .contains("kit_sidebar_item @18,18 236x44")
    );
    let label = app.find(text_query("Acme")).bounds;
    assert_eq!(label.origin.x, 18.0 + 10.0 + 22.0 + 10.0);
    assert_eq!(label.size.height, 18.0);
    assert!(app.layout_snapshot().contains("svg @28,29 22x22"));
}

#[test]
fn labels_start_after_the_icon_slot_and_subitems_indent_thirty_pixels() {
    let mut app = TestApp::new(sidebar_demo(Theme::dark()));
    let layout = app.layout_snapshot();
    assert!(layout.contains("text[3] @56,"), "{layout}");
    assert!(layout.contains("pl-30"), "{layout}");
    assert!(layout.contains("text[2] @48,"), "{layout}");
}

#[test]
fn a_group_header_pins_a_sixteen_pixel_chevron_ten_pixels_from_the_edge() {
    let mut app = TestApp::new(sidebar_demo(Theme::dark()));
    assert!(
        app.layout_snapshot().contains("svg[4] @228,"),
        "{}",
        app.layout_snapshot()
    );
    assert!(
        app.frame_snapshot()
            .contains("stroke1.6 path[3] 6,9 12x6 #8a8a93ff")
    );
}

#[test]
fn clicking_a_row_selects_it_and_reports_it_checked() {
    let mut app = TestApp::new(sidebar_demo(Theme::dark()));
    assert_eq!(
        app.find(role(Role::MenuItem).name("Inbox")).states.checked,
        Some(true)
    );
    assert_eq!(
        app.find(role(Role::MenuItem).name("Tasks")).states.checked,
        Some(false)
    );
    app.click(role(Role::MenuItem).name("Tasks"));
    assert_eq!(
        app.find(role(Role::MenuItem).name("Inbox")).states.checked,
        Some(false)
    );
    assert_eq!(
        app.find(role(Role::MenuItem).name("Tasks")).states.checked,
        Some(true)
    );
}

#[test]
fn enter_activates_the_focused_row() {
    let mut app = TestApp::new(sidebar_demo(Theme::dark()));
    app.focus(role(Role::MenuItem).name("Tasks"));
    assert!(app.find(role(Role::MenuItem).name("Tasks")).states.focused);
    app.press_key("Enter");
    assert_eq!(
        app.find(role(Role::MenuItem).name("Tasks")).states.checked,
        Some(true)
    );
}

#[test]
fn a_disabled_row_ignores_clicks_and_does_not_wash() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let hits = cx.signal(0);
        sidebar(
            (
                sidebar_item("Archive", icons::INBOX, false)
                    .disabled(true)
                    .on_click(move |cx, _| hits.update(cx, |hits| *hits += 1)),
                text(bind(move |runtime| format!("hits {}", hits.get(runtime)))),
            ),
            (),
        )
    }));
    let archive = item_bounds(&mut app, "Archive").center();
    assert!(
        app.find(role(Role::MenuItem).name("Archive"))
            .states
            .disabled
    );
    app.pointer_move(archive.x, archive.y);
    app.pointer_down(archive.x, archive.y);
    app.pointer_up(archive.x, archive.y);
    app.settle();
    assert!(!app.frame_snapshot().contains(HOVER_FILL));
    assert!(app.exists(text_query("hits 0")));
}

#[test]
fn hovering_washes_an_idle_row_with_the_palettes_hover_color() {
    let mut app = TestApp::new(sidebar_demo(Theme::dark()));
    assert!(!app.frame_snapshot().contains(HOVER_FILL));
    app.hover(role(Role::MenuItem).name("Tasks"));
    app.advance_ms(40);
    assert!(app.is_animating());
    assert!(app.frame_snapshot().contains("opacity0."));
    app.settle();
    assert!(app.frame_snapshot().contains(&fill_row(HOVER_FILL)));
}

#[test]
fn a_selected_row_shows_the_pill_and_hairline_and_ignores_hover() {
    let mut app = TestApp::new(sidebar_demo(Theme::dark()));
    let frame = app.frame_snapshot();
    assert!(frame.contains(&fill_row(PILL_FILL)), "{frame}");
    assert!(frame.contains(PILL_BORDER), "{frame}");
    app.hover(role(Role::MenuItem).name("Inbox"));
    app.settle();
    assert!(!app.frame_snapshot().contains(HOVER_FILL));
}

#[test]
fn selection_fades_the_pill_and_the_icon_color_in_with_a_spring() {
    let mut app = TestApp::new(sidebar_demo(Theme::dark()));
    let idle_icon = "stroke1.8 path[6] 2,2 20x20 #8a8a93ff";
    let selected_icon = "stroke1.8 path[6] 2,2 20x20 #fafafaff";
    assert!(app.frame_snapshot().contains(idle_icon));
    app.click(role(Role::MenuItem).name("Tasks"));
    app.advance_ms(40);
    let mid = app.frame_snapshot();
    assert!(app.is_animating());
    assert!(
        mid.contains(idle_icon) && mid.contains(selected_icon),
        "{mid}"
    );
    assert!(mid.contains("opacity0."), "{mid}");
    app.settle();
    let settled = app.frame_snapshot();
    assert!(
        settled.contains(selected_icon) && !settled.contains(idle_icon),
        "{settled}"
    );
    assert_eq!(settled.matches(PILL_BORDER).count(), 2, "{settled}");
}

#[test]
fn the_light_palette_paints_its_translucent_pill_without_a_visible_border() {
    let app = TestApp::new(sidebar_demo(Theme::light()));
    let frame = app.frame_snapshot();
    assert!(frame.contains(&fill_row("#b0b0b080")), "{frame}");
    assert!(!frame.contains("stroke1 0.5,0.5 235x35"), "{frame}");
}

#[test]
fn switching_themes_repaints_the_pill() {
    let mut app = TestApp::new(component(|cx| {
        let theme = provide_theme(cx, Theme::dark());
        vstack_of(theme)
    }));
    assert!(app.frame_snapshot().contains(&fill_row(PILL_FILL)));
    app.click(role(Role::Button).name("Switch"));
    app.settle();
    let frame = app.frame_snapshot();
    assert!(frame.contains(&fill_row("#b0b0b080")), "{frame}");
    assert!(!frame.contains(&fill_row(PILL_FILL)), "{frame}");
}

fn vstack_of(theme: Signal<Theme>) -> impl View {
    widgets::vstack((
        button("Switch").on_click(move |cx, _| theme.set(cx, Theme::light())),
        sidebar_item("Inbox", icons::INBOX, true),
    ))
    .w(236.0)
}

#[test]
fn toggling_a_group_header_collapses_its_body_with_a_spring() {
    let mut app = TestApp::new(sidebar_demo(Theme::dark()));
    assert_eq!(clip_height(&app.frame_snapshot()), 76.0);
    let settings = item_bounds(&mut app, "Settings");
    app.click(role(Role::Button).name("Finance"));
    app.advance_ms(60);
    assert!(app.is_animating());
    let mid = clip_height(&app.frame_snapshot());
    assert!(mid > 0.0 && mid < 76.0, "{mid}");
    app.settle();
    assert_eq!(clip_height(&app.frame_snapshot()), 0.0);
    assert_eq!(
        item_bounds(&mut app, "Settings").origin.y,
        settings.origin.y - 76.0
    );
    app.click(role(Role::Button).name("Finance"));
    app.settle();
    assert_eq!(clip_height(&app.frame_snapshot()), 76.0);
}

#[test]
fn rows_below_a_collapsing_group_follow_it_with_the_same_spring() {
    let mut app = TestApp::new(sidebar_demo(Theme::dark()));
    let before = app.frame_snapshot();
    app.click(role(Role::Button).name("Finance"));
    app.advance_ms(60);
    assert_ne!(app.frame_snapshot(), before);
    assert!(
        app.frame_snapshot().contains("layer at 0,"),
        "{}",
        app.frame_snapshot()
    );
}

#[test]
fn a_group_header_reads_as_a_button_and_checked_while_open() {
    let mut app = TestApp::new(sidebar_demo(Theme::dark()));
    assert_eq!(
        app.find(role(Role::Button).name("Finance")).states.checked,
        Some(true)
    );
    app.click(role(Role::Button).name("Finance"));
    assert_eq!(
        app.find(role(Role::Button).name("Finance")).states.checked,
        Some(false)
    );
}

#[test]
fn the_section_caption_is_a_twenty_six_pixel_inset_label_that_is_not_a_row() {
    let mut app = TestApp::new(sidebar_demo(Theme::dark()));
    let caption = app.find(text_query("Personal")).bounds;
    assert_eq!(caption.origin.x, 18.0 + 10.0);
    assert!((caption.size.height - 14.4).abs() < 0.01);
    assert!(!app.exists(role(Role::MenuItem).name("Personal")));
    assert!(
        app.layout_snapshot()
            .contains("kit_sidebar_section[1] > stack @18,68 236x26")
    );
    assert!(
        app.layout_snapshot()
            .contains("kit_sidebar_section[1] > stack[1] @18,96 236x116 gap-4/4")
    );
}

#[test]
fn the_separator_is_a_twelve_pixel_gap_with_a_centered_hairline() {
    let mut app = TestApp::new(sidebar_demo(Theme::dark()));
    let separator = app.find(role(Role::Separator)).bounds;
    assert_eq!((separator.size.width, separator.size.height), (236.0, 12.0));
    let frame = app.frame_snapshot();
    assert!(frame.contains("layer at 0,5.5\n"), "{frame}");
    assert!(frame.contains("fill 0,0 236x1 #262628ff"), "{frame}");
}

#[test]
fn the_shell_is_a_260_wide_menu_beside_a_rounded_bordered_card() {
    let mut app = TestApp::new(sidebar_demo(Theme::dark()));
    let layout = app.layout_snapshot();
    assert!(
        layout.contains("kit_sidebar_menu @6,6 260x588 p-12 gap-6/6"),
        "{layout}"
    );
    assert!(
        layout.contains("kit_sidebar_card[1] @266,6 528x588"),
        "{layout}"
    );
    let frame = app.frame_snapshot();
    assert!(frame.contains("fill 0,0 528x588 r16 #0f0f10cc"), "{frame}");
    assert!(
        frame.contains("stroke1 0.5,0.5 527x587 r15.5 #27272aff"),
        "{frame}"
    );
}

#[test]
fn a_header_row_hugs_the_window_top_and_the_card_by_the_layout_padding() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        sidebar_with_header(
            sidebar_item("Inbox", icons::INBOX, true),
            (button("Search"), spacer(), button("New")),
            text("content"),
        )
    }));
    let search = bounds(&mut app, Role::Button, "Search");
    let new = bounds(&mut app, Role::Button, "New");
    assert_eq!((search.origin.x, search.origin.y), (266.0, 6.0));
    assert_eq!(new.origin.x + new.size.width, 794.0);
    let content = app.find(text_query("content")).bounds;
    assert_eq!(content.origin.y, 6.0 + 34.0 + 6.0);
}

fn search_header(width: f32) -> TestApp {
    TestApp::with_size(
        component(|cx| {
            provide_theme(cx, Theme::dark());
            let query = cx.signal(String::new());
            let picked = cx.signal(String::from("none"));
            let rows = cx.signal(vec!["Inbox".into(), "Tasks".into()]);
            sidebar_with_header(
                sidebar_item("Inbox", icons::INBOX, true),
                (
                    spacer(),
                    sidebar_search(query)
                        .suggestions(rows)
                        .on_select(move |cx, index| picked.set(cx, format!("#{index}"))),
                    spacer(),
                ),
                text(bind(move |runtime| {
                    format!("picked={}", picked.get(runtime))
                })),
            )
        }),
        width,
        600.0,
    )
}

#[test]
fn a_header_search_between_spacers_is_centered_at_its_reading_width() {
    let mut app = search_header(1400.0);
    let bar = app.find(role(Role::SearchField)).bounds;
    let content_side = 1388.0 - 272.0;
    assert_eq!(
        (bar.origin.x, bar.origin.y, bar.size.width, bar.size.height),
        (272.0 + (content_side - 320.0) / 2.0, 6.0, 320.0, 34.0)
    );
}

#[test]
fn a_header_search_gives_way_on_a_narrow_content_side() {
    let mut app = search_header(560.0);
    let bar = app.find(role(Role::SearchField)).bounds;
    let (leading, trailing) = (bar.origin.x - 272.0, 548.0 - bar.origin.x - bar.size.width);
    assert!(bar.size.width < 320.0, "{bar:?}");
    assert_eq!(leading, trailing);
}

#[test]
fn a_header_search_springs_wider_while_it_is_open() {
    let mut app = search_header(1400.0);
    assert_eq!(app.find(role(Role::SearchField)).bounds.size.width, 320.0);
    app.focus(role(Role::SearchField));
    app.advance_ms(60);
    let opening = app.find(role(Role::SearchField)).bounds.size.width;
    assert!(opening > 320.0 && opening < 480.0, "{opening}");
    app.settle();
    assert_eq!(app.find(role(Role::SearchField)).bounds.size.width, 480.0);
}

#[test]
fn a_header_search_offers_its_suggestions_and_reports_the_pick() {
    let mut app = search_header(800.0);
    app.focus(role(Role::SearchField));
    app.settle();
    assert_eq!(app.count(role(Role::ListItem)), 2);
    app.click(role(Role::ListItem).name("Tasks"));
    assert_eq!(app.text_of(text_query("picked=")), "picked=#1");
}

#[test]
fn every_state_of_the_sidebar_paints_the_palette() {
    let mut app = TestApp::new(sidebar_demo(Theme::dark()));
    app.hover(role(Role::MenuItem).name("Tasks"));
    app.settle();
    insta::assert_snapshot!(app.frame_snapshot());
}

#[test]
fn a_group_that_starts_closed_is_collapsed_from_the_first_frame() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let open = cx.signal(false);
        sidebar(
            sidebar_group(
                "Finance",
                icons::HOUSE,
                open,
                sidebar_column(sidebar_subitem("Local", false)),
            ),
            (),
        )
    }));
    assert_eq!(clip_height(&app.frame_snapshot()), 0.0);
    assert!(!app.is_animating());
    app.click(role(Role::Button).name("Finance"));
    app.settle();
    assert_eq!(clip_height(&app.frame_snapshot()), 36.0);
}
