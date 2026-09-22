use super::previews::Fixture;
use super::*;
use omega::testing::{Drawn, State, SystemTopic};
use omega_proto::omega::{Capability, EventKind};

#[test]
fn the_center_lists_raised_notifications() {
    let drawn = Drawn::of::<Center>(&Fixture::raised()).unwrap();

    assert!(drawn.text().contains("Battery low"));
    assert!(drawn.text().contains("15% remaining"));
    assert!(drawn.text().contains("Update ready"));
    assert!(drawn.text().contains("2 ACTIVE"));
}

#[test]
fn the_center_shows_an_empty_state_when_nothing_is_raised() {
    let drawn = Drawn::of::<Center>(&State::new().absent(SystemTopic::Notifications)).unwrap();

    assert!(drawn.first("status").is_some());
}

#[test]
fn the_manifest_declares_battery_events_and_the_notify_capability() {
    let manifest = plugin().manifest().unwrap();

    assert!(
        manifest
            .events
            .contains(&(EventKind::EventBatteryLow as i32))
    );
    assert!(
        manifest
            .events
            .contains(&(EventKind::EventBatteryCritical as i32))
    );
    assert!(manifest.granted().unwrap().contains(&Capability::Notify));
}
