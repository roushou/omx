use super::*;
use omega::testing::{
    State, SystemTopic,
    topic::{BacklightState, MonitorsState},
};
use omega_preview::Cases;

pub(crate) struct Fixture;

impl Fixture {
    pub(crate) fn monitors(count: usize) -> MonitorsState {
        let mut state = MonitorsState::default();
        state.monitors.resize_with(count, Default::default);

        for (index, monitor) in state.monitors.iter_mut().enumerate() {
            monitor.id = if index == 0 { "eDP-1" } else { "DP-1" }.into();
            monitor.connected = true;
            monitor.width = if index == 0 { 2880 } else { 3840 };
            monitor.height = if index == 0 { 1800 } else { 2160 };
            monitor.refresh_mhz = 60000;
            monitor.scale = if index == 0 { 2.0 } else { 1.5 };
        }

        state
    }

    pub(crate) fn state() -> State {
        State::new()
            .with(Self::monitors(2))
            .with(BacklightState { percent: 60 })
    }
}

#[test]
fn preview() {
    Cases::new()
        .surface::<Indicator>("bar", Fixture::state())
        .surface::<Panel>("displays", Fixture::state())
        .surface::<Panel>(
            "external-only",
            Fixture::state().absent(SystemTopic::Backlight),
        )
        .surface::<Panel>(
            "unavailable",
            State::new()
                .absent(SystemTopic::Backlight)
                .absent(SystemTopic::Monitors),
        )
        .run()
        .unwrap();
}
