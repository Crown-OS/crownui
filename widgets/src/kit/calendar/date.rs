/// A day of the proleptic Gregorian calendar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CalendarDate {
    /// Year, e.g. 2026.
    pub year: i32,
    /// Month, 1 to 12.
    pub month: u8,
    /// Day of the month, 1 to 31.
    pub day: u8,
}

/// Weeks in the grid, fixed so a calendar keeps its height while it is paged.
pub(super) const WEEKS: usize = 6;
pub(super) const CELLS: usize = WEEKS * 7;

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

impl CalendarDate {
    /// The date `year`-`month`-`day`, clamped into a real day.
    pub fn new(year: i32, month: u8, day: u8) -> Self {
        let month = month.clamp(1, 12);
        Self {
            year,
            month,
            day: day.clamp(1, days_in_month(year, month)),
        }
    }

    /// Days since 1970-01-01, negative before it.
    pub fn days_since_epoch(self) -> i64 {
        let year = i64::from(self.year) - i64::from(self.month <= 2);
        let era = year.div_euclid(400);
        let year_of_era = year.rem_euclid(400);
        let month = i64::from(self.month);
        let day_of_year =
            (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + i64::from(self.day) - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        era * 146_097 + day_of_era - 719_468
    }

    /// The date `days` after 1970-01-01.
    pub fn from_days_since_epoch(days: i64) -> Self {
        let shifted = days + 719_468;
        let era = shifted.div_euclid(146_097);
        let day_of_era = shifted.rem_euclid(146_097);
        let year_of_era =
            (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let month_index = (5 * day_of_year + 2) / 153;
        let day = day_of_year - (153 * month_index + 2) / 5 + 1;
        let month = if month_index < 10 {
            month_index + 3
        } else {
            month_index - 9
        };
        let year = year_of_era + era * 400 + i64::from(month <= 2);
        Self {
            year: year as i32,
            month: month as u8,
            day: day as u8,
        }
    }

    /// Monday 0 to Sunday 6.
    pub fn weekday_from_monday(self) -> u8 {
        (self.days_since_epoch() + 3).rem_euclid(7) as u8
    }

    /// Whether it falls on a Saturday or a Sunday.
    pub fn is_weekend(self) -> bool {
        self.weekday_from_monday() >= 5
    }

    /// The first of this date's month moved by whole `months`.
    pub fn first_of_month_shifted(self, months: i32) -> Self {
        let index = i64::from(self.year) * 12 + i64::from(self.month) - 1 + i64::from(months);
        Self {
            year: index.div_euclid(12) as i32,
            month: (index.rem_euclid(12) + 1) as u8,
            day: 1,
        }
    }

    /// "September 2026".
    pub fn month_title(self) -> String {
        format!("{} {}", MONTH_NAMES[usize::from(self.month - 1)], self.year)
    }

    /// The day in `cell` of the six-week, Monday-first grid around this date's month.
    pub(super) fn grid_cell(self, cell: usize) -> Self {
        let first = self.first_of_month_shifted(0);
        let start = first.days_since_epoch() - i64::from(first.weekday_from_monday());
        Self::from_days_since_epoch(start + cell as i64)
    }
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_days_round_trip_across_leap_years_and_centuries() {
        for days in [-800_000, -1, 0, 1, 10_957, 19_723, 20_000, 1_000_000] {
            assert_eq!(
                CalendarDate::from_days_since_epoch(days).days_since_epoch(),
                days
            );
        }
        assert_eq!(CalendarDate::new(1970, 1, 1).days_since_epoch(), 0);
        assert_eq!(CalendarDate::new(2000, 3, 1).days_since_epoch(), 11_017);
    }

    #[test]
    fn weekdays_count_from_monday() {
        assert_eq!(CalendarDate::new(1970, 1, 1).weekday_from_monday(), 3);
        assert_eq!(CalendarDate::new(2026, 10, 5).weekday_from_monday(), 0);
        assert!(CalendarDate::new(2026, 10, 4).is_weekend());
    }

    #[test]
    fn a_grid_starts_on_the_monday_before_the_first_and_covers_the_month() {
        for (year, month) in [(2026, 2), (2026, 3), (2026, 8), (2024, 2), (2027, 5)] {
            let anchor = CalendarDate::new(year, month, 15);
            let cells: Vec<_> = (0..CELLS).map(|cell| anchor.grid_cell(cell)).collect();
            assert_eq!(cells[0].weekday_from_monday(), 0);
            assert!(cells.contains(&CalendarDate::new(year, month, 1)));
            assert!(cells.contains(&CalendarDate::new(year, month, days_in_month(year, month))));
        }
    }

    #[test]
    fn paging_crosses_years_both_ways() {
        let january = CalendarDate::new(2026, 1, 31);
        assert_eq!(
            january.first_of_month_shifted(-1),
            CalendarDate::new(2025, 12, 1)
        );
        assert_eq!(
            january.first_of_month_shifted(13),
            CalendarDate::new(2027, 2, 1)
        );
        assert_eq!(january.month_title(), "January 2026");
    }
}
