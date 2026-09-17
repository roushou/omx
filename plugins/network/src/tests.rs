use super::fixtures::Fixture;
use super::*;
use omega::{
    effect::EffectError,
    surface::{Lifecycle, TextEdit},
    testing::{
        Drawn, SurfaceHarness, manifest_of,
        operation::{Action, Operation, Refusal},
        topic::WifiState,
    },
};

#[tokio::test]
async fn selected_ssid_and_password_produce_one_request_and_failure_is_visible() {
    let mut panel = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();
    let other = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();
    let drawn = panel.draw();
    panel
        .interact(&drawn, "networks", "activate", "Guest/5G~".to_string())
        .unwrap();
    let drawn = panel.draw();
    let reset = panel.model().password.reset_revision();
    panel
        .interact(
            &drawn,
            "password",
            "change",
            TextEdit {
                text: "fixture-secret".into(),
                revision: 1,
                reset,
            },
        )
        .unwrap();

    assert!(other.model().password.text().is_empty());

    let drawn = panel.draw();
    panel.interact(&drawn, "connect", "press", ()).unwrap();
    panel.send(Message::Connect).unwrap();

    assert!(panel.model().requesting);
    assert!(panel.model().password.text().is_empty());

    let drawn = panel.draw();

    assert_eq!(drawn.flag("connect", "disabled"), Some(true));

    let op = panel
        .complete_effect(Err(EffectError::Refused(Refusal::unavailable(
            "fixture refused connection",
        ))))
        .await
        .unwrap();

    assert!(matches!(
        op, Operation::Act(act)
            if matches!(
                act.action.as_ref().unwrap().kind.as_ref(), Some(Action::ConnectWifi(input))
                    if input.ssid == "Guest/5G~" && input.password == "fixture-secret"
            )
    ));

    panel.complete().await.unwrap();

    assert!(!panel.model().requesting);
    assert!(panel.draw().text().contains("fixture refused connection"));
    assert!(panel.take_effect().is_none());
}

#[tokio::test]
async fn admission_does_not_claim_connection_success() {
    let mut panel = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();
    panel.send(Message::Select("Cafe".into())).unwrap();
    panel.send(Message::Connect).unwrap();
    panel.complete_effect(Ok(None)).await.unwrap();
    panel.complete().await.unwrap();

    assert!(!panel.model().requesting);

    let drawn = panel.draw();

    assert!(drawn.text().starts_with("Home"));
    assert!(!drawn.text().contains("Connected to Cafe"));
    assert!(panel.take_effect().is_none());
}

#[tokio::test]
async fn hidden_panels_and_changed_selections_clear_credentials() {
    let mut panel = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();

    for lifecycle in [Lifecycle::Hidden, Lifecycle::Closed] {
        panel.send(Message::Select("Guest/5G~".into())).unwrap();
        let reset = panel.model().password.reset_revision();
        panel
            .send(Message::Password(TextEdit {
                text: "fixture-secret".into(),
                revision: 1,
                reset,
            }))
            .unwrap();

        assert!(!panel.model().password.text().is_empty());

        panel.lifecycle(lifecycle).unwrap();

        assert!(panel.model().password.text().is_empty());
        assert!(panel.model().selected.is_none());
    }
}

#[tokio::test]
async fn disappearing_networks_are_rejected_before_effects() {
    let mut panel = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();
    panel.send(Message::Select("Guest/5G~".into())).unwrap();
    panel.state(&Fixture::state().with(WifiState::default()));
    panel.send(Message::Connect).unwrap();

    assert!(panel.draw().text().contains("no longer available"));
    assert!(panel.take_effect().is_none());
}

#[tokio::test]
async fn disconnect_is_wifi_only_and_does_not_repeat() {
    let mut panel = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();
    panel.send(Message::Disconnect).unwrap();
    panel.send(Message::Disconnect).unwrap();
    let op = panel.complete_effect(Ok(None)).await.unwrap();

    assert!(matches!(
        op, Operation::Act(act)
            if matches!(
                act.action.as_ref().unwrap().kind.as_ref(), Some(Action::DisconnectWifi(_))
            )
    ));

    panel.complete().await.unwrap();

    assert!(panel.take_effect().is_none());
}

#[test]
fn vpn_and_interface_traffic_are_explicitly_named() {
    let drawn = Drawn::of::<Panel>(&Fixture::vpn()).unwrap();

    assert!(drawn.text().contains("VPN"));
    assert!(drawn.text().contains("Work"));
    assert!(drawn.text().contains("wg0"));
    assert!(drawn.text().contains("wlan0"));
    assert!(drawn.text().contains("Receiving"));
}

#[test]
fn no_process_or_record_capabilities_are_requested() {
    let grants = manifest_of(&plugin()).granted().unwrap();

    assert!(grants.contains(&omega::internal::Capability::Network));
    assert!(!grants.contains(&omega::internal::Capability::Spawn));
    assert!(!grants.contains(&omega::internal::Capability::StateWrite));
}
