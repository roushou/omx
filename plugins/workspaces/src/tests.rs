use super::{fixtures::Fixture, *};
use omega::platform::desktop::WorkspaceIndex;
use omega::{
    Args, Input,
    config::{Fields, IntoValue},
    testing::{
        Called, Drawn, State, SurfaceHarness, SystemTopic, manifest_of,
        operation::{Action, Operation},
    },
};
use omega_proto::omega::{Capability, switch_workspace};

#[test]
fn default_slots_match_omarchy_and_bind_typed_destinations() {
    let drawn = Drawn::of::<Indicator>(&Fixture::at(3)).unwrap();
    let keys: Vec<_> = drawn
        .keys()
        .into_iter()
        .filter(|key| key.starts_with("workspace-"))
        .collect();

    assert_eq!(
        keys,
        [
            "workspace-1",
            "workspace-2",
            "workspace-3",
            "workspace-4",
            "workspace-5",
            "workspace-10"
        ]
    );

    assert_eq!(drawn.prop("workspace-3", "label").as_deref(), Some("●"));
    assert_eq!(drawn.prop("workspace-10", "label").as_deref(), Some("0"));
    assert_eq!(
        drawn.prop("workspace-2", "emphasis").as_deref(),
        Some("muted")
    );

    assert_eq!(
        drawn.prop("workspace-3", "emphasis").as_deref(),
        Some("primary")
    );

    assert!(
        drawn
            .prop("workspace-3", "tooltip")
            .unwrap()
            .contains("Code · DP-1 · 2 windows")
    );

    for key in keys {
        assert_eq!(drawn.flag(key, "flat"), Some(true));
        let binding = &drawn.node(key).unwrap().events["press"];

        assert_eq!(binding.command, "workspaces.select");
        assert_eq!(
            WorkspaceIndex::decode(Args::new(binding.args.clone()))
                .unwrap()
                .get()
                .to_string(),
            key.strip_prefix("workspace-").unwrap()
        );
    }
}

#[test]
fn empty_readings_keep_persistent_buttons_but_absence_has_no_actions() {
    let drawn = Drawn::of::<Indicator>(&Fixture::empty()).unwrap();

    assert_eq!(
        drawn
            .keys()
            .iter()
            .filter(|key| key.starts_with("workspace-"))
            .count(),
        5
    );

    let absent = Drawn::of::<Indicator>(&State::new().absent(SystemTopic::Workspaces)).unwrap();

    assert!(absent.first("button").is_none());
}

#[test]
fn focus_and_hotplug_readings_update_without_local_selection_state() {
    let mut harness = SurfaceHarness::<Indicator>::new(&Fixture::at(1)).unwrap();

    assert_eq!(
        harness.draw().prop("workspace-1", "label").as_deref(),
        Some("●")
    );

    harness.state(&Fixture::at(3));
    let next = harness.draw();

    assert_eq!(next.prop("workspace-1", "label").as_deref(), Some("1"));
    assert_eq!(next.prop("workspace-3", "label").as_deref(), Some("●"));
    harness.state(&Fixture::empty());
    let empty = harness.draw();

    assert!(empty.node("workspace-10").is_none());
    assert_eq!(empty.prop("workspace-3", "label").as_deref(), Some("3"));
}

#[test]
fn settings_are_bounded_and_vertical_layout_is_explicit() {
    let settings = Settings {
        persistent: 255,
        maximum: 0,
        vertical: true,
    };

    let drawn = Drawn::configured::<Indicator>(&Fixture::at(1), &settings.write()).unwrap();

    assert_eq!(
        drawn
            .keys()
            .iter()
            .filter(|key| key.starts_with("workspace-"))
            .count(),
        1
    );

    assert_eq!(drawn.prop("root", "align").as_deref(), Some("column"));
    let settings = Settings {
        persistent: 0,
        maximum: 10,
        vertical: false,
    };

    let drawn = Drawn::configured::<Indicator>(&Fixture::empty(), &settings.write()).unwrap();

    assert!(drawn.first("button").is_none());
}

#[tokio::test]
async fn selection_can_open_an_unobserved_workspace_and_rejects_bad_input() {
    let called = Called::of::<Select>(&State::new(), WorkspaceIndex::new(5).unwrap()).await;

    assert!(called.answer.is_ok());
    let [Operation::Act(act)] = called.effects.as_slice() else {
        panic!("expected one action")
    };

    let Some(Action::SwitchWorkspace(switch)) = act.action.as_ref().and_then(|a| a.kind.as_ref())
    else {
        panic!("expected workspace switch")
    };

    assert_eq!(switch.target, Some(switch_workspace::Target::Index(5)));

    for value in [
        0_u64.into_value(),
        "next".into_value(),
        (-1_i64).into_value(),
    ] {
        let called = Called::raw::<Select>(&State::new(), vec![value]).await;

        assert!(called.answer.is_err());
        assert!(called.effects.is_empty());
    }
}

#[test]
fn the_manifest_requests_readings_without_process_execution() {
    let manifest = manifest_of(&plugin());

    assert_eq!(manifest.state_topics, ["workspaces"]);
    assert!(!manifest.capabilities.contains(&(Capability::Spawn as i32)));
}
