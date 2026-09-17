use super::{Settings, reading::Reading};
use omega::{
    Surface, View,
    platform::time::Clock,
    surface::{Events, Task},
    ui::Text,
};
use std::convert::Infallible;

/// Local weekday and time, refreshed by the daemon once per minute.
#[derive(Debug, omega::Surface)]
pub struct Indicator {
    clock: Clock,
    #[omega(config)]
    settings: Settings,
}

impl Surface for Indicator {
    type Model = ();
    type Message = Infallible;
    type Effects = ();

    fn render(&self, _: &(), _: &Events<Infallible>) -> View {
        match Reading::of(&self.clock) {
            Some(reading) => Text::new(reading.label(&self.settings))
                .tooltip(reading.tooltip(&self.settings))
                .into(),
            None => Text::new("—").muted().tooltip("Clock unavailable").into(),
        }
    }

    fn update(&self, _: &mut (), message: Infallible, _: &()) -> Task<Infallible> {
        match message {}
    }
}
