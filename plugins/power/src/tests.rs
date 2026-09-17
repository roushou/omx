use super::previews::Fixture;
use super::*;
use omega::testing::{Called, Drawn, SystemTopic, topic::BatteryState};

#[test]
fn low_charge_warns_only_while_unplugged() {
    for mains in [false, true] {
        let panel =
            Drawn::of::<Panel>(&Fixture::state().battery(0.08, false).mains(mains)).unwrap();

        assert_eq!(
            panel.prop("charge", "tone").as_deref(),
            if mains { None } else { Some("warning") }
        );
    }

    let charging = Drawn::of::<Panel>(&Fixture::state().battery(0.08, true)).unwrap();

    assert!(charging.prop("charge", "tone").is_none());
}

#[test]
fn unavailable_battery_does_not_appear_empty_or_hide_profiles() {
    let panel = Drawn::of::<Panel>(&Fixture::state().absent(SystemTopic::Battery)).unwrap();

    assert!(panel.text().contains("BATTERY UNAVAILABLE"));
    assert!(!panel.text().contains("0%"));
    assert!(panel.node("charge").is_none());
    assert!(panel.text().contains("Balanced"));
}

#[test]
fn time_and_full_charge_have_correct_meanings() {
    let panel = Drawn::of::<Panel>(&Fixture::state()).unwrap();

    assert!(panel.text().contains("3h 10m"));

    let charging = Fixture::state().with(BatteryState {
        level: 0.42,
        charging: true,
        seconds_to_full: 2400,
        seconds_to_empty: 0,
    });
    let panel = Drawn::of::<Panel>(&charging).unwrap();

    assert!(panel.text().contains("Time to full"));
    assert!(panel.text().contains("40m"));

    let full = Drawn::of::<Indicator>(&Fixture::state().battery(1.0, false).mains(true)).unwrap();

    assert_eq!(
        full.prop(&full.first("icon").unwrap(), "name").as_deref(),
        Some("󰂅")
    );
}

#[tokio::test]
async fn profile_selection_revalidates_current_availability() {
    let state = Fixture::state().power_profile(PowerProfile::Balanced, [PowerProfile::Balanced]);

    for rejected in [PowerProfile::Saver, PowerProfile::Unspecified] {
        let called = Called::of::<ChangeProfile>(&state, rejected).await;

        assert!(called.answer.is_err());
        assert!(called.effects.is_empty());
    }

    let called = Called::of::<ChangeProfile>(&state, PowerProfile::Balanced).await;

    assert!(called.answer.is_ok());
    assert_eq!(called.effects.len(), 1);
}
