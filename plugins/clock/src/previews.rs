use super::{Indicator, Message, Panel, Settings, fixtures::Fixture};
use omega::{
    config::Fields,
    testing::{State, SurfaceHarness, SystemTopic},
};
use omega_preview::Cases;

#[test]
fn preview() {
    Cases::new()
        .surface::<Indicator>("bar", Fixture::at(2026, 9, 16, 9, 5))
        .surface::<Panel>("calendar", Fixture::at(2026, 9, 16, 9, 5))
        .surface::<Panel>("leap-year", Fixture::at(2024, 2, 29, 12, 0))
        .surface::<Panel>("iso-week-boundary", Fixture::at(2021, 1, 1, 0, 0))
        .surface::<Panel>("unavailable", State::new().absent(SystemTopic::Time))
        .surface_with::<Panel>("browsing", || {
            let mut panel = SurfaceHarness::new(&Fixture::at(2026, 9, 16, 9, 5))?;
            panel.send(Message::Shift(1))?;
            Ok(panel)
        })
        .surface_with::<Panel>("sunday-start", || {
            SurfaceHarness::configured(
                &Fixture::at(2026, 9, 16, 9, 5),
                &Settings {
                    monday_first: false,
                    twelve_hour: true,
                    ..Settings::default()
                }
                .write(),
            )
        })
        .run()
        .unwrap();
}
