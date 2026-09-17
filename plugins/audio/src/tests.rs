use super::previews::Fixture;
use super::*;
use omega::{
    config::IntoValue,
    testing::{
        Called, Drawn, State, SystemTopic,
        operation::{Action, Operation},
    },
};
use omega_proto::omega::set_volume;

#[test]
fn unavailable_audio_has_no_controls() {
    let panel = Drawn::of::<Panel>(&State::new().absent(SystemTopic::Audio)).unwrap();

    assert!(panel.text().contains("Audio unavailable"));
    assert!(panel.first("slider").is_none());
    assert!(panel.first("toggle").is_none());
}

#[test]
fn controls_use_current_volume_and_mute_state() {
    for muted in [false, true] {
        let panel = Drawn::of::<Panel>(&Fixture::output(muted)).unwrap();

        assert!(panel.text().contains("55%"));

        let toggle = panel.first("toggle").unwrap();

        assert_eq!(panel.flag(&toggle, "on"), Some(!muted));
        assert_eq!(
            panel.node(&toggle).unwrap().events["change"].command,
            "audible"
        );

        assert_eq!(
            panel.node("output/slider").unwrap().events["change"].command,
            "volume"
        );
    }
}

#[tokio::test]
async fn input_and_availability_are_checked_before_effects() {
    for value in [-0.1, 1.1, f64::NAN] {
        let called =
            Called::raw::<SetVolume>(&Fixture::output(false), vec![value.into_value()]).await;

        assert!(called.answer.is_err());
        assert!(called.effects.is_empty());
    }

    let missing =
        Called::of::<SetVolume>(&State::new().absent(SystemTopic::Audio), Percent::whole(60)).await;

    assert!(missing.answer.is_err());
    assert!(missing.effects.is_empty());

    let valid = Called::of::<SetVolume>(&Fixture::output(false), Percent::whole(60)).await;

    assert!(valid.answer.is_ok());
    assert_eq!(valid.effects.len(), 1);
}

#[tokio::test]
async fn repeated_mute_requests_send_absolute_state_despite_stale_readings() {
    for observed_muted in [false, true] {
        let state = Fixture::output(observed_muted);

        for audible in [true, true, false, false] {
            let called = Called::of::<SetAudible>(&state, audible).await;

            assert!(called.answer.is_ok());
            let [Operation::Act(act)] = called.effects.as_slice() else {
                panic!("expected one audio action");
            };

            let Some(Action::SetVolume(volume)) = act.action.as_ref().and_then(|a| a.kind.as_ref())
            else {
                panic!("expected a volume action");
            };

            assert_eq!(volume.change, Some(set_volume::Change::Muted(!audible)));
        }
    }
}

#[tokio::test]
async fn unavailable_audio_refuses_mute_requests_without_effects() {
    let state = State::new().absent(SystemTopic::Audio);

    for audible in [true, false] {
        let called = Called::of::<SetAudible>(&state, audible).await;

        assert!(called.answer.is_err());
        assert!(called.effects.is_empty());
    }
}
