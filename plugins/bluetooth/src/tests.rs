use super::{previews::Fixture, *};
use omega::{
    config::IntoValue,
    testing::{
        Called, Drawn, State, SystemTopic, manifest_of,
        operation::{Action, Operation},
        topic::BluetoothState,
    },
};

#[test]
fn adapter_states_are_distinct_and_missing_batteries_are_not_zero() {
    for (state, text) in [
        (
            State::new().absent(SystemTopic::Bluetooth),
            "Bluetooth unavailable",
        ),
        (
            State::new().with(BluetoothState::default()),
            "No Bluetooth adapter",
        ),
        (
            State::new().with(BluetoothState {
                available: true,
                ..Default::default()
            }),
            "Bluetooth is off",
        ),
    ] {
        let panel = Drawn::of::<Panel>(&state).unwrap();

        assert!(panel.text().contains(text));
        assert!(panel.first("button").is_none());
    }

    let text = Drawn::of::<Panel>(&Fixture::state()).unwrap().text();

    assert!(text.contains("Battery 76%"));
    assert!(text.contains("Battery 0%"));
    assert_eq!(text.matches("Battery").count(), 2);
}

#[tokio::test]
async fn controls_bind_adapter_qualified_endpoints() {
    let state = Fixture::state();
    let id = DeviceId::try_from("/org/bluez/hci1/dev_AA_BB_CC_DD_EE_01").unwrap();
    let called = Called::of::<Connect>(&state, id.clone()).await;

    assert!(called.answer.is_ok());
    assert_eq!(called.effects.len(), 1);
    assert!(matches!(
        &called.effects[0], Operation::Act(act)
            if matches!(
                act.action.as_ref().unwrap().kind.as_ref(), Some(Action::ConnectBluetooth(input))
                    if input.device_id == id.as_str()
            )
    ));

    let panel = Drawn::of::<Panel>(&state).unwrap();
    let button = panel
        .tree()
        .root
        .as_ref()
        .unwrap()
        .children
        .iter()
        .flat_map(|n| n.children.iter())
        .find(|n| {
            n.events
                .get("press")
                .is_some_and(|b| b.command == "connect")
        })
        .unwrap();

    assert_eq!(button.events["press"].args, vec![id.into_value()]);
}

#[tokio::test]
async fn stale_invalid_and_unavailable_connections_are_refused() {
    let id = DeviceId::try_from("/org/bluez/hci1/dev_AA_BB_CC_DD_EE_01").unwrap();
    let mut blocked = Fixture::devices();
    blocked.devices[1].can_connect = false;

    for state in [
        State::new().absent(SystemTopic::Bluetooth),
        State::new().with(blocked),
    ] {
        let called = Called::of::<Connect>(&state, id.clone()).await;

        assert!(called.answer.is_err());
        assert!(called.effects.is_empty());
    }

    let invalid = Called::raw::<Connect>(
        &Fixture::state(),
        vec!["AA:BB:CC:DD:EE:01".to_string().into_value()],
    )
    .await;

    assert!(invalid.answer.is_err());
    assert!(invalid.effects.is_empty());
}

#[tokio::test]
async fn matching_state_does_not_issue_a_toggle() {
    let connected = DeviceId::try_from("/org/bluez/hci0/dev_AA_BB_CC_DD_EE_01").unwrap();
    let disconnected = DeviceId::try_from("/org/bluez/hci1/dev_AA_BB_CC_DD_EE_01").unwrap();

    assert!(
        Called::of::<Connect>(&Fixture::state(), connected.clone())
            .await
            .effects
            .is_empty()
    );

    assert!(
        Called::of::<Disconnect>(&Fixture::state(), disconnected)
            .await
            .effects
            .is_empty()
    );

    let called = Called::of::<Disconnect>(&Fixture::state(), connected).await;

    assert!(called.answer.is_ok());
    assert_eq!(called.effects.len(), 1);
}

#[test]
fn capabilities_are_limited_to_bluetooth() {
    assert_eq!(
        manifest_of(&plugin()).granted().unwrap(),
        vec![
            omega::internal::Capability::StateRead,
            omega::internal::Capability::Bluetooth
        ]
    );
}
