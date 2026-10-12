use crowntest::{Role, TestApp, role};
use view::component;
use widgets::kit::{CalendarDate, Theme, calendar, provide_theme, quick_panel};

fn october() -> TestApp {
    TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let today = cx.signal(CalendarDate::new(2026, 10, 5));
        quick_panel(calendar(today))
    }))
}

#[test]
fn it_opens_on_todays_month_and_pages_both_ways() {
    let mut app = october();
    assert!(app.exists(role(Role::Heading).name("October 2026")));
    app.click(role(Role::Button).name("Next month"));
    assert!(app.exists(role(Role::Heading).name("November 2026")));
    app.click(role(Role::Button).name("Previous month"));
    app.click(role(Role::Button).name("Previous month"));
    assert!(app.exists(role(Role::Heading).name("September 2026")));
}

#[test]
fn six_weeks_keep_the_grid_one_height() {
    let mut app = october();
    let layout = app.layout_snapshot();
    assert!(layout.contains("kit_calendar @0,8 296x252"), "{layout}");
}

#[test]
fn a_calendar_composes_its_grid() {
    let mut app = october();
    app.settle();
    insta::assert_snapshot!(app.snapshot());
}
