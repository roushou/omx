use chrono::{Datelike, Days, Months, NaiveDate, Weekday};

/// A browsable month, restricted to years 1–9999 so the surrounding grid stays valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Month(NaiveDate);

impl Month {
    pub(crate) fn of(date: NaiveDate) -> Self {
        assert!((1..=9999).contains(&date.year()), "supported calendar year");
        Self(date.with_day(1).expect("every month has a first day"))
    }

    pub(crate) fn title(self) -> String {
        self.0.format("%B %Y").to_string()
    }

    pub(crate) fn shift(self, delta: i32) -> Option<Self> {
        let amount = Months::new(delta.unsigned_abs());
        let date = if delta < 0 {
            self.0.checked_sub_months(amount)?
        } else {
            self.0.checked_add_months(amount)?
        };
        (1..=9999).contains(&date.year()).then_some(Self(date))
    }

    pub(crate) fn contains(self, date: NaiveDate) -> bool {
        date.year() == self.0.year() && date.month() == self.0.month()
    }

    pub(crate) fn weeks(self, monday_first: bool) -> [Week; 6] {
        let leading = if monday_first {
            self.0.weekday().num_days_from_monday()
        } else {
            self.0.weekday().num_days_from_sunday()
        };

        let first = self
            .0
            .checked_sub_days(Days::new(u64::from(leading)))
            .expect("bounded calendar leaves room for the leading week");
        std::array::from_fn(|row| {
            let days: [NaiveDate; 7] = std::array::from_fn(|column| {
                first
                    .checked_add_days(Days::new((row * 7 + column) as u64))
                    .expect("bounded calendar leaves room for six weeks")
            });
            let thursday = days
                .iter()
                .find(|day| day.weekday() == Weekday::Thu)
                .expect("every week contains Thursday");
            Week {
                number: thursday.iso_week().week(),
                days,
            }
        })
    }
}

#[derive(Debug)]
pub(crate) struct Week {
    pub(crate) number: u32,
    pub(crate) days: [NaiveDate; 7],
}

pub(crate) struct Year;

impl Year {
    pub(crate) fn progress(today: NaiveDate) -> f64 {
        let first = NaiveDate::from_ymd_opt(today.year(), 1, 1).expect("valid year");
        let next = NaiveDate::from_ymd_opt(today.year() + 1, 1, 1).expect("bounded year");
        f64::from(today.ordinal0()) / (next - first).num_days() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gregorian_leap_years_and_contiguous_six_week_grids() {
        for (year, days_in_february) in [(1900, 28), (2000, 29), (2024, 29), (2100, 28)] {
            let month = Month::of(NaiveDate::from_ymd_opt(year, 2, 1).unwrap());

            for monday in [false, true] {
                let grid = month.weeks(monday);
                let days: Vec<_> = grid.iter().flat_map(|week| week.days).collect();

                assert_eq!(days.len(), 42);
                assert_eq!(
                    days.iter().filter(|day| month.contains(**day)).count(),
                    days_in_february
                );

                assert_eq!(
                    days[0].weekday(),
                    if monday { Weekday::Mon } else { Weekday::Sun }
                );

                assert!(
                    days.windows(2)
                        .all(|pair| (pair[1] - pair[0]).num_days() == 1)
                );
            }
        }
    }

    #[test]
    fn iso_week_numbers_belong_to_the_rows_thursday() {
        let month = Month::of(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap());

        for monday in [true, false] {
            let weeks = month.weeks(monday);

            assert_eq!(weeks[0].number, 53);
            assert_eq!(weeks[1].number, 1);
        }
    }

    #[test]
    fn navigation_crosses_years_and_stops_at_bounds() {
        let january = Month::of(NaiveDate::from_ymd_opt(2026, 1, 31).unwrap());

        assert_eq!(january.shift(-1).unwrap().title(), "December 2025");
        assert_eq!(january.shift(12).unwrap().title(), "January 2027");
        assert!(
            Month::of(NaiveDate::from_ymd_opt(1, 1, 1).unwrap())
                .shift(-1)
                .is_none()
        );

        assert!(
            Month::of(NaiveDate::from_ymd_opt(9999, 12, 1).unwrap())
                .shift(1)
                .is_none()
        );

        for year in [1, 9999] {
            assert_eq!(
                Month::of(NaiveDate::from_ymd_opt(year, 12, 1).unwrap())
                    .weeks(true)
                    .len(),
                6
            );
        }
    }

    #[test]
    fn year_progress_counts_completed_days_in_leap_years() {
        assert_eq!(
            Year::progress(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            0.0
        );

        assert_eq!(
            Year::progress(NaiveDate::from_ymd_opt(2024, 7, 2).unwrap()),
            0.5
        );

        assert_eq!(
            Year::progress(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap()),
            364.0 / 365.0
        );
    }
}
