use super::{previews::Fixture, *};
use omega::{
    config::IntoValue,
    testing::{
        Called, Drawn, State, SurfaceHarness, SystemTopic, manifest_of,
        operation::{Action, Operation},
        topic::MediaState,
    },
};

#[test]
fn empty_players_hide_the_bar_and_keep_an_explanation_in_the_panel() {
    for state in [
        State::new().absent(SystemTopic::Media),
        State::new().with(MediaState::default()),
    ] {
        assert!(Drawn::of::<Indicator>(&state).unwrap().is_empty());
        assert!(
            Drawn::of::<Panel>(&state)
                .unwrap()
                .first("button")
                .is_none()
        );
    }
}

#[test]
fn selection_is_local_and_transport_targets_the_selected_player() {
    let mut panel = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();
    let mut other = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();
    let drawn = panel.draw();
    panel
        .interact(
            &drawn,
            "players",
            "activate",
            "firefox.instance42".to_string(),
        )
        .unwrap();
    let drawn = panel.draw();

    assert!(drawn.text().contains("A quiet afternoon"));
    assert!(other.draw().text().contains("Midnight City"));
    assert_eq!(
        drawn.node("playback").unwrap().events["press"].command,
        "media.play"
    );

    assert_eq!(
        drawn.node("playback").unwrap().events["press"].args,
        vec!["firefox.instance42".to_string().into_value()]
    );

    assert_eq!(drawn.flag("next", "disabled"), Some(true));
    assert_eq!(drawn.flag("previous", "disabled"), Some(true));
    assert!(panel.take_effect().is_none());
}

#[test]
fn disappearing_selection_does_not_redirect_controls_to_another_player() {
    let mut panel = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();
    panel
        .send(Message::Select("firefox.instance42".into()))
        .unwrap();
    let mut media = Fixture::players();
    media.players.pop();
    panel.state(&State::new().with(media));
    let drawn = panel.draw();

    assert!(drawn.text().contains("selected player closed"));
    assert!(drawn.first("button").is_none());

    panel.send(Message::Select("spotify".into())).unwrap();

    assert!(panel.draw().text().contains("Midnight City"));
}

#[tokio::test]
async fn unsupported_and_closed_players_produce_no_effect() {
    for state in [Fixture::state(), State::new().with(MediaState::default())] {
        let called =
            Called::of::<Next>(&state, PlayerId::try_from("firefox.instance42").unwrap()).await;

        assert!(called.answer.is_err());
        assert!(called.effects.is_empty());
    }

    let mut media = Fixture::players();
    media.players[0].can_control = false;
    let called = Called::of::<Pause>(
        &State::new().with(media),
        PlayerId::try_from("spotify").unwrap(),
    )
    .await;

    assert!(called.answer.is_err());
    assert!(called.effects.is_empty());
}

#[tokio::test]
async fn playback_commands_always_target_the_bound_player() {
    let called = Called::of::<Play>(
        &Fixture::state(),
        PlayerId::try_from("firefox.instance42").unwrap(),
    )
    .await;

    assert!(called.answer.is_ok());
    assert_eq!(called.effects.len(), 1);
    assert!(matches!(
        &called.effects[0], Operation::Act(act)
            if matches!(
                act.action.as_ref().unwrap().kind.as_ref(), Some(Action::MediaKey(input))
                    if input.player_id.as_deref() == Some("firefox.instance42")
            )
    ));
}

#[test]
fn paused_players_remain_accessible_and_unicode_titles_are_bounded() {
    let mut media = Fixture::players();
    media.players.remove(0);

    assert!(
        !Drawn::of::<Indicator>(&State::new().with(media))
            .unwrap()
            .is_empty()
    );

    let title = "音楽のある暮らしを楽しむ毎日です";
    let short = Players::shorten(title, 8);

    assert_eq!(short.chars().count(), 8);
    assert!(short.ends_with('…'));
    assert_eq!(
        manifest_of(&plugin()).granted().unwrap(),
        vec![omega::internal::Capability::StateRead]
    );
}

#[test]
fn command_effects_belong_to_the_host() {
    let ui = manifest_of(&plugin());
    assert!(ui.commands.is_empty());
    let host = media_commands::Host::declaration().manifest().unwrap();
    assert!(
        host.granted()
            .unwrap()
            .contains(&omega::internal::Capability::Media)
    );
    assert!(!host.commands.is_empty());
}
