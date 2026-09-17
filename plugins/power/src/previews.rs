use super::*;
use omega::testing::{State, SystemTopic, topic::BatteryState};
use omega_preview::Cases;

pub(crate) struct Fixture;

impl Fixture {
    pub(crate) fn state() -> State {
        State::new()
            .with(BatteryState {
                level: 0.72,
                charging: false,
                seconds_to_empty: 11400,
                seconds_to_full: 0,
            })
            .mains(false)
            .power_profile(
                PowerProfile::Balanced,
                [
                    PowerProfile::Saver,
                    PowerProfile::Balanced,
                    PowerProfile::Performance,
                ],
            )
    }
}

#[test]
fn preview() {
    Cases::new()
        .surface::<Indicator>("bar", Fixture::state())
        .surface::<Panel>("battery", Fixture::state())
        .surface::<Panel>(
            "charging",
            Fixture::state().mains(true).with(BatteryState {
                level: 0.42,
                charging: true,
                seconds_to_full: 2400,
                seconds_to_empty: 0,
            }),
        )
        .surface::<Panel>("low", Fixture::state().battery(0.08, false))
        .surface::<Panel>(
            "desktop",
            Fixture::state().absent(SystemTopic::Battery).mains(true),
        )
        .surface::<Panel>(
            "unavailable",
            State::new()
                .absent(SystemTopic::Battery)
                .absent(SystemTopic::Mains)
                .absent(SystemTopic::PowerProfile),
        )
        .run()
        .unwrap();
}
