use super::{Indicator, Panel};
use omega::testing::{
    State, SystemTopic,
    topic::{AudioState, AudioStream, AudioStreamsState},
};
use omega_preview::Cases;

pub(crate) struct Fixture;

impl Fixture {
    pub(crate) fn output(muted: bool) -> State {
        State::new()
            .with(AudioState {
                volume: 0.55,
                muted,
                ..Default::default()
            })
            .absent(SystemTopic::AudioStreams)
    }

    pub(crate) fn streams() -> State {
        Self::output(false).with(AudioStreamsState {
            streams: vec![
                AudioStream {
                    index: 1,
                    app: "vlc".into(),
                    volume: 0.4,
                    muted: false,
                },
                AudioStream {
                    index: 2,
                    app: "chromium".into(),
                    volume: 0.8,
                    muted: true,
                },
            ],
        })
    }
}

#[test]
fn preview() {
    Cases::new()
        .surface::<Indicator>("bar", Fixture::output(false))
        .surface::<Panel>("output", Fixture::output(false))
        .surface::<Panel>("muted", Fixture::output(true))
        .surface::<Panel>("streams", Fixture::streams())
        .surface::<Panel>(
            "unavailable",
            State::new()
                .absent(SystemTopic::Audio)
                .absent(SystemTopic::AudioStreams),
        )
        .run()
        .unwrap();
}
