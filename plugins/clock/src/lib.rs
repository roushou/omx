//! A local clock and calendar. Time comes from Omega; browsing belongs to each panel.

mod calendar;
mod indicator;
mod panel;
mod reading;

pub use indicator::Indicator;
pub use panel::{Message, Model, Panel};

/// Display preferences shared by the bar indicator and calendar panel.
#[derive(Debug, Clone, omega::Config)]
pub struct Settings {
    /// Use a 12-hour clock with AM/PM. Defaults to false.
    pub twelve_hour: bool,
    /// Include the full weekday in the bar. Defaults to true.
    pub show_weekday: bool,
    /// Start calendar rows on Monday; false selects Sunday. Defaults to true.
    pub monday_first: bool,
    /// Include ISO week numbers, numbered by each row's Thursday. Defaults to true.
    pub show_week_numbers: bool,
    /// Show the share of the current year completed. Defaults to true.
    pub show_year_progress: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            twelve_hour: false,
            show_weekday: true,
            monday_first: true,
            show_week_numbers: true,
            show_year_progress: true,
        }
    }
}

/// Register the indicator and calendar. This plugin only reads the time topic.
pub fn plugin() -> omega::Plugin {
    omega::plugin!().surface(Indicator).surface(Panel)
}

#[cfg(test)]
mod fixtures;

#[cfg(test)]
mod previews;

#[cfg(test)]
mod tests;
