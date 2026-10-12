mod date;
mod kind;
mod metrics;

pub use date::CalendarDate;
pub use kind::CalendarKind;

use reactive::Signal;
use view::{Element, Semantic};

/// A month grid opened on `today`'s month: six Monday-first weeks under the month's name, today on
/// an accent disc, neighbouring months' days dimmed, and two arrows that page a month at a time.
/// The grid keeps its height whatever the month, so it never resizes under the pointer.
pub fn calendar(today: Signal<CalendarDate>) -> Element<CalendarKind> {
    Element::new(CalendarKind { today }).label("Calendar")
}
