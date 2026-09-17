use super::{fixtures::Fixture, *};
use omega::{
    config::Fields,
    effect::EffectError,
    testing::{
        Drawn, State, SurfaceHarness, SystemTopic, manifest_of,
        operation::{Action, Operation, PresentationAction, Refusal},
        topic::ApplicationsState,
    },
};

struct Inspect;

impl Inspect {
    fn ids(panel: &mut SurfaceHarness<Panel>) -> Vec<String> {
        panel
            .draw()
            .node("results")
            .unwrap()
            .children
            .iter()
            .map(|child| child.key.clone())
            .collect()
    }

    fn id() -> ApplicationId {
        ApplicationId::try_from("org.example.Files.desktop").unwrap()
    }

    fn activate(panel: &mut SurfaceHarness<Panel>) {
        let drawn = panel.draw();
        panel
            .interact(&drawn, "results", "activate", Self::id())
            .unwrap();
    }
}

#[test]
fn empty_and_unavailable_catalogues_are_distinct() {
    for (state, text) in [
        (
            State::new().absent(SystemTopic::Applications),
            "Application catalogue unavailable",
        ),
        (
            State::new().with(ApplicationsState::default()),
            "No installed applications",
        ),
    ] {
        let drawn = Drawn::of::<Panel>(&state).unwrap();

        assert!(drawn.text().contains(text));
        assert!(drawn.node("results").unwrap().children.is_empty());
    }

    assert!(!Drawn::of::<Indicator>(&State::new()).unwrap().is_empty());
}

#[test]
fn search_matches_all_tokens_and_ranks_names_before_metadata() {
    let mut panel = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();

    for (query, id) in [
        ("  FiLe  folder  ", "org.example.Files.desktop"),
        ("shell console", "org.example.Terminal.desktop"),
        ("websites", "org.example.Browser.desktop"),
        ("ÉDITEUR 日本語", "org.example.Unicode.desktop"),
    ] {
        Fixture::edit(&mut panel, query);

        assert_eq!(Inspect::ids(&mut panel), vec![id]);
    }

    Fixture::edit(&mut panel, "file web");

    assert!(Inspect::ids(&mut panel).is_empty());
    assert!(panel.draw().text().contains("No matching applications"));

    let mut catalogue = Fixture::catalogue();
    catalogue.applications[0].keywords.push("terminal".into());
    panel.state(&State::new().with(catalogue));
    Fixture::edit(&mut panel, "terminal");

    assert_eq!(
        Inspect::ids(&mut panel),
        vec!["org.example.Terminal.desktop", "org.example.Files.desktop"]
    );
}

#[test]
fn ordering_and_limits_are_stable_with_duplicate_labels() {
    let mut catalogue = Fixture::catalogue();
    catalogue.applications[0].name = "Same".into();
    catalogue.applications[1].name = "Same".into();
    let mut panel = SurfaceHarness::<Panel>::new(&State::new().with(catalogue.clone())).unwrap();
    Fixture::edit(&mut panel, "same");
    let before = Inspect::ids(&mut panel);
    catalogue.applications.reverse();
    panel.state(&State::new().with(catalogue));

    assert_eq!(Inspect::ids(&mut panel), before);

    let settings = Settings {
        max_results: 0,
        ..Default::default()
    }
    .write();
    let mut limited = SurfaceHarness::<Panel>::configured(&Fixture::state(), &settings).unwrap();

    assert_eq!(Inspect::ids(&mut limited).len(), 1);
    assert!(limited.draw().text().contains("Showing 1 of 8"));
}

#[test]
fn query_selection_and_clears_are_instance_local_and_revision_aware() {
    let mut first = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();
    let mut second = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();
    Fixture::edit(&mut first, "files");

    assert_eq!(Inspect::ids(&mut first).len(), 1);
    assert_eq!(Inspect::ids(&mut second).len(), 8);

    let old = TextEdit {
        text: "terminal".into(),
        revision: first.model().query.revision() + 1,
        reset: first.model().query.reset_revision(),
    };

    first.send(Message::Clear).unwrap();
    first.send(Message::Edited(old)).unwrap();

    assert!(first.model().query.text().is_empty());

    let drawn = first.draw();
    first
        .interact(&drawn, "results", "select", Inspect::id())
        .unwrap();

    assert_eq!(first.model().selected.as_ref(), Some(&Inspect::id()));
    assert!(second.model().selected.is_none());
}

#[test]
fn field_navigation_and_escape_bindings_are_declared() {
    let mut panel = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();
    let drawn = panel.draw();

    assert_eq!(drawn.node("query-0").unwrap().navigation_target, "results");
    assert_eq!(drawn.flag("query-0", "autofocus"), Some(true));
    assert!(
        drawn
            .node("results")
            .unwrap()
            .events
            .contains_key("activate")
    );

    assert_eq!(drawn.tree().root.as_ref().unwrap().shortcuts.len(), 1);

    panel.lifecycle(Lifecycle::Presented).unwrap();

    assert!(panel.draw().node("query-1").is_some());
}

#[tokio::test]
async fn activation_is_single_admission_then_hide() {
    let mut panel = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();
    Inspect::activate(&mut panel);
    panel.send(Message::Activate(Inspect::id())).unwrap();

    assert!(panel.model().pending);

    let drawn = panel.draw();

    assert_eq!(drawn.flag("results", "disabled"), Some(true));

    let operation = panel.complete_effect(Ok(None)).await.unwrap();

    assert!(matches!(
        operation, Operation::Act(act)
            if matches!(
                act.action.as_ref().unwrap().kind.as_ref(), Some(Action::LaunchApp(input))
                    if input.desktop_id == "org.example.Files.desktop" && input.uris.is_empty()
            )
    ));

    assert!(panel.take_effect().is_none());

    panel.complete().await.unwrap();
    let recorded = panel.complete_effect(Ok(None)).await.unwrap();
    assert!(matches!(recorded, Operation::SetState(_)));
    assert!(panel.model().pending);
    panel.complete().await.unwrap();
    let hide = panel.complete_effect(Ok(None)).await.unwrap();

    assert!(matches!(
        hide, Operation::ChangePresentation(change)
            if change.action == PresentationAction::Hide as i32
    ));

    panel.complete().await.unwrap();

    assert!(panel.take_effect().is_none());
}

#[tokio::test]
async fn launch_refusal_is_visible_and_does_not_dismiss_or_retry() {
    let mut panel = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();
    Fixture::edit(&mut panel, "files");
    Inspect::activate(&mut panel);
    panel
        .complete_effect(Err(EffectError::Refused(Refusal::unavailable(
            "Fixture launch refused",
        ))))
        .await
        .unwrap();
    panel.complete().await.unwrap();

    assert!(!panel.model().pending);
    assert!(panel.draw().text().contains("Fixture launch refused"));
    assert_eq!(panel.model().query.text(), "files");
    assert!(panel.take_effect().is_none());
}

#[tokio::test]
async fn stale_and_filtered_targets_are_refused_before_activation() {
    let mut panel = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();
    Fixture::edit(&mut panel, "terminal");
    panel.send(Message::Activate(Inspect::id())).unwrap();

    assert!(panel.take_effect().is_none());

    panel.send(Message::Clear).unwrap();
    let mut catalogue = Fixture::catalogue();
    catalogue.applications.remove(0);
    panel.state(&State::new().with(catalogue));
    panel.send(Message::Activate(Inspect::id())).unwrap();

    assert!(
        panel
            .draw()
            .text()
            .contains("no longer in the current results")
    );

    assert!(panel.take_effect().is_none());

    let drawn = panel.draw();

    assert!(
        panel
            .interact(&drawn, "results", "activate", "/bin/sh".to_string())
            .is_err()
    );

    assert!(panel.take_effect().is_none());
}

#[tokio::test]
async fn late_admission_does_not_hide_a_reopened_panel() {
    let mut panel = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();
    Inspect::activate(&mut panel);
    panel.lifecycle(Lifecycle::Hidden).unwrap();
    panel.lifecycle(Lifecycle::Presented).unwrap();

    assert!(panel.model().pending);

    panel.complete_effect(Ok(None)).await.unwrap();
    panel.complete().await.unwrap();

    let recorded = panel.complete_effect(Ok(None)).await.unwrap();
    assert!(matches!(recorded, Operation::SetState(_)));
    panel.complete().await.unwrap();
    assert!(!panel.model().pending);
    assert!(panel.take_effect().is_none());
}

#[tokio::test]
async fn escape_hides_only_this_instance_and_reports_a_refusal() {
    let mut panel = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();
    panel.send(Message::Dismiss).unwrap();
    panel.send(Message::Dismiss).unwrap();
    let operation = panel
        .complete_effect(Err(EffectError::Refused(Refusal::unavailable(
            "Fixture hide refused",
        ))))
        .await
        .unwrap();

    assert!(matches!(
        operation, Operation::ChangePresentation(change)
            if change.instance.as_ref().unwrap().id == "fixture"
    ));

    panel.complete().await.unwrap();

    assert!(
        panel
            .draw()
            .text()
            .contains("Could not dismiss the launcher")
    );

    assert!(panel.take_effect().is_none());
}

#[test]
fn hide_and_close_clear_the_query_and_selection() {
    let mut panel = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();

    for lifecycle in [Lifecycle::Hidden, Lifecycle::Closed] {
        Fixture::edit(&mut panel, "files");
        panel.send(Message::Selected(Inspect::id())).unwrap();
        panel.lifecycle(lifecycle).unwrap();

        assert!(panel.model().query.text().is_empty());
        assert!(panel.model().selected.is_none());
    }

    assert_eq!(
        manifest_of(&plugin()).granted().unwrap(),
        vec![
            omega::internal::Capability::StateRead,
            omega::internal::Capability::StateWrite,
            omega::internal::Capability::Spawn
        ]
    );
}

#[test]
fn fuzzy_search_handles_subsequences_and_preserves_literal_priority() {
    let mut panel = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();
    Fixture::edit(&mut panel, "txted");
    assert_eq!(Inspect::ids(&mut panel), vec!["org.example.Editor.desktop"]);
    Fixture::edit(&mut panel, "trmnl");
    assert_eq!(
        Inspect::ids(&mut panel),
        vec!["org.example.Terminal.desktop"]
    );
    Fixture::edit(&mut panel, "日本");
    assert_eq!(
        Inspect::ids(&mut panel),
        vec!["org.example.Unicode.desktop"]
    );
    Fixture::edit(&mut panel, "zzzz");
    assert!(Inspect::ids(&mut panel).is_empty());
}

#[test]
fn history_orders_favorites_then_recents_and_bounds_admissions() {
    use omega::record::PluginState;
    let mut history = History::default();
    let files = Inspect::id();
    let terminal = ApplicationId::try_from("org.example.Terminal.desktop").unwrap();
    history.launched(&files);
    history.launched(&terminal);
    history.launched(&files);
    assert_eq!(
        history.recent,
        vec![files.to_string(), terminal.to_string()]
    );
    history.toggle(&terminal);
    let state = Fixture::state().keyspace(&History::address(), history.write());
    let mut panel = SurfaceHarness::<Panel>::new(&state).unwrap();
    assert_eq!(
        &Inspect::ids(&mut panel)[..2],
        &[terminal.to_string(), files.to_string()]
    );
    Fixture::edit(&mut panel, "files");
    assert_eq!(Inspect::ids(&mut panel)[0], files.to_string());
    history.toggle(&terminal);
    assert!(!history.favorite(&terminal));
    for i in 0..30 {
        history.launched(&ApplicationId::try_from(format!("app-{i}.desktop")).unwrap());
    }
    assert_eq!(history.recent.len(), 20);
    assert_eq!(history.recent[0], "app-29.desktop");
}

#[tokio::test]
async fn favorite_button_publishes_without_launching_or_dismissing() {
    let mut panel = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();
    Fixture::edit(&mut panel, "files");
    let drawn = panel.draw();
    panel.interact(&drawn, "favorite", "press", ()).unwrap();
    let operation = panel.complete_effect(Ok(None)).await.unwrap();
    assert!(matches!(operation, Operation::SetState(_)));
    panel.complete().await.unwrap();
    assert!(panel.take_effect().is_none());
}

#[tokio::test]
async fn history_failure_is_visible_without_replaying_the_launch() {
    let mut panel = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();
    Inspect::activate(&mut panel);
    panel.complete_effect(Ok(None)).await.unwrap();
    panel.complete().await.unwrap();
    panel.send(Message::Activate(Inspect::id())).unwrap();
    panel
        .complete_effect(Err(EffectError::Refused(Refusal::unavailable(
            "record refused",
        ))))
        .await
        .unwrap();
    panel.complete().await.unwrap();
    assert!(!panel.model().pending);
    assert!(
        panel
            .draw()
            .text()
            .contains("Could not retain launcher history")
    );
    assert!(panel.take_effect().is_none());
}

#[test]
fn search_keeps_results_height_and_footer_nodes_stable() {
    let mut panel = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();
    let first = panel.draw();
    let height = first.node("results").unwrap().props.get("height").cloned();
    let keys: Vec<_> = first
        .tree()
        .root
        .as_ref()
        .unwrap()
        .children
        .iter()
        .map(|node| node.key.clone())
        .collect();
    for query in ["files", "zzzz", "", "txted"] {
        Fixture::edit(&mut panel, query);
        let drawn = panel.draw();
        assert_eq!(
            drawn.node("results").unwrap().props.get("height").cloned(),
            height
        );
        let current: Vec<_> = drawn
            .tree()
            .root
            .as_ref()
            .unwrap()
            .children
            .iter()
            .map(|node| node.key.clone())
            .collect();
        assert_eq!(current, keys);
        assert_eq!(drawn.flag("favorite", "disabled"), Some(query == "zzzz"));
    }
}
