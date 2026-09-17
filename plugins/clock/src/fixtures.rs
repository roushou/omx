use chrono::{Datelike, NaiveDate};
use omega::testing::{State, topic::TimeState};

pub(crate) struct Fixture;

impl Fixture {
    pub(crate) fn at(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> State {
        let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
        State::new().with(TimeState {
            unix_seconds: date
                .and_hms_opt(hour, minute, 0)
                .unwrap()
                .and_utc()
                .timestamp(),
            zone: "UTC".into(),
            utc_offset_seconds: 0,
            year,
            month,
            day,
            hour,
            minute,
            weekday: date.weekday().num_days_from_sunday(),
        })
    }
}
