use super::Settings;
use chrono::{NaiveDate, NaiveTime};
use omega::platform::time::Clock;

pub(crate) struct Reading {
    pub(crate) date: NaiveDate,
    pub(crate) time: NaiveTime,
    pub(crate) zone: String,
}

impl Reading {
    pub(crate) fn of(clock: &Clock) -> Option<Self> {
        if !clock.has_reading() || !(1..=9999).contains(&clock.year()) {
            return None;
        }

        Some(Self {
            date: NaiveDate::from_ymd_opt(clock.year(), clock.month(), clock.day())?,
            time: NaiveTime::from_hms_opt(clock.hour(), clock.minute(), 0)?,
            zone: clock.zone(),
        })
    }

    pub(crate) fn time(&self, settings: &Settings) -> String {
        self.time
            .format(if settings.twelve_hour {
                "%-I:%M %p"
            } else {
                "%H:%M"
            })
            .to_string()
    }

    pub(crate) fn label(&self, settings: &Settings) -> String {
        if settings.show_weekday {
            format!("{} {}", self.date.format("%A"), self.time(settings))
        } else {
            self.time(settings)
        }
    }

    pub(crate) fn tooltip(&self, settings: &Settings) -> String {
        format!(
            "{} · {} {}",
            self.date.format("%A, %-d %B %Y"),
            self.time(settings),
            self.zone
        )
        .trim_end()
        .to_owned()
    }
}
