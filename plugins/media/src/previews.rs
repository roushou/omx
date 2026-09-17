use super::*;
use omega::testing::{State, SurfaceHarness, SystemTopic, topic::MediaState};
use omega_preview::Cases;

pub(crate) struct Fixture;

impl Fixture {
    pub(crate) fn players() -> MediaState {
        let mut media = MediaState::default();
        media.players.push(Default::default());
        let p = &mut media.players[0];
        p.id = "spotify".into();
        p.identity = "Spotify".into();
        p.playback = Playback::Playing as i32;
        p.title = "Midnight City".into();
        p.artist = "M83".into();
        p.album = "Hurry Up, We're Dreaming".into();
        p.length_us = 244_000_000;
        p.active = true;
        p.can_control = true;
        p.can_play = true;
        p.can_pause = true;
        p.can_go_next = true;
        p.can_go_previous = true;
        media.players.push(Default::default());
        let p = &mut media.players[1];
        p.id = "firefox.instance42".into();
        p.identity = "Firefox".into();
        p.playback = Playback::Paused as i32;
        p.title = "A quiet afternoon".into();
        p.artist = "Radio".into();
        p.can_control = true;
        p.can_play = true;
        p.can_pause = true;
        media
    }

    pub(crate) fn state() -> State {
        State::new().with(Self::players())
    }
}

#[test]
fn preview() {
    Cases::new()
        .surface::<Indicator>("bar", Fixture::state())
        .surface::<Panel>("players", Fixture::state())
        .surface_with::<Panel>("paused", || {
            let mut panel = SurfaceHarness::new(&Fixture::state())?;
            panel.send(Message::Select("firefox.instance42".into()))?;
            Ok(panel)
        })
        .surface::<Panel>("empty", State::new().with(MediaState::default()))
        .surface::<Panel>("unavailable", State::new().absent(SystemTopic::Media))
        .run()
        .unwrap();
}
