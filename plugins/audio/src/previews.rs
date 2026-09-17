use super::{Indicator, Panel};
use omega::testing::{State, SystemTopic, topic::AudioState};
use omega_preview::Cases;

pub(crate) struct Fixture;

impl Fixture {
    pub(crate) fn output(muted: bool) -> State {
        State::new().with(AudioState {
            volume: 0.55,
            muted,
            ..Default::default()
        })
    }
}

#[test]
fn preview() {
    Cases::new()
        .surface::<Indicator>("bar", Fixture::output(false))
        .surface::<Panel>("output", Fixture::output(false))
        .surface::<Panel>("muted", Fixture::output(true))
        .surface::<Panel>("unavailable", State::new().absent(SystemTopic::Audio))
        .run()
        .unwrap();
}
