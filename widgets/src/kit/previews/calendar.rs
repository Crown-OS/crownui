use view::{View, component};

use crate::kit::{
    CalendarDate, calendar,
    preview::{Preview, stage},
    quick_panel,
};

pub(super) fn previews() -> Vec<Preview> {
    vec![Preview {
        name: "calendar_october",
        build: || {
            component(|cx| {
                let today = cx.signal(CalendarDate::new(2026, 10, 5));
                stage(quick_panel(calendar(today)))
            })
            .boxed()
        },
    }]
}
