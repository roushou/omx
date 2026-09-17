use super::{previews::Fixture, *};
use omega::{
    config::Fields,
    testing::{
        Drawn, State, manifest_of,
        topic::{SystemState, ThermalsState},
    },
};

#[test]
fn absent_services_do_not_invent_zero_readings() {
    let panel = Drawn::of::<Panel>(&Fixture::missing()).unwrap();
    let text = panel.text();

    assert!(text.contains("CPU and memory unavailable"));
    assert!(text.contains("Storage readings unavailable"));
    assert!(text.contains("Thermal readings unavailable"));
    assert!(!text.contains("0%"));
    assert!(panel.first("progress").is_none());
}

#[test]
fn memory_uses_available_bytes_and_readings_remain_labelled() {
    let text = Drawn::of::<Panel>(&Fixture::state()).unwrap().text();

    for expected in [
        "6.0 GiB / 16.0 GiB",
        "coretemp · Package",
        "nvme · Composite",
        "2200 RPM",
        "0.72 / 0.54 / 0.38",
    ] {
        assert!(text.contains(expected), "{text}");
    }
}

#[test]
fn missing_mounts_duplicates_and_no_sensors_are_explicit() {
    let settings = Settings {
        mounts: vec!["/".into(), "/".into(), "/missing".into()],
        ..Default::default()
    }
    .write();
    let state = Fixture::state().with(ThermalsState::default());
    let panel = Drawn::configured::<Panel>(&state, &settings).unwrap();

    assert!(panel.text().contains("/missing Not reported"));
    assert!(panel.text().contains("No readable sensors"));
    assert_eq!(panel.text().matches("btrfs").count(), 1);
}

#[test]
fn zero_swap_and_inconsistent_memory_do_not_produce_invalid_percentages() {
    let state = Fixture::state().with(SystemState {
        memory_total_bytes: 100,
        memory_available_bytes: 200,
        ..Default::default()
    });
    let panel = Drawn::of::<Panel>(&state).unwrap();

    assert!(panel.text().contains("Swap Not configured"));

    let system = SystemState {
        memory_total_bytes: 0,
        ..Default::default()
    };

    let bar = Drawn::of::<Indicator>(&State::new().with(system)).unwrap();

    assert_eq!(bar.text(), "0%");
}

#[test]
fn per_core_readings_are_opt_in_and_plugin_has_no_effects() {
    assert!(
        !Drawn::of::<Panel>(&Fixture::state())
            .unwrap()
            .text()
            .contains("Core 0")
    );

    let settings = Settings {
        show_cores: true,
        ..Default::default()
    }
    .write();

    assert!(
        Drawn::configured::<Panel>(&Fixture::state(), &settings)
            .unwrap()
            .text()
            .contains("Core 0")
    );

    assert_eq!(
        manifest_of(&plugin()).granted().unwrap(),
        vec![omega::internal::Capability::StateRead]
    );
}
