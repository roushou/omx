use super::{fixtures::Fixture, *};
use omega::{
    config::Fields,
    effect::EffectError,
    testing::{
        Drawn, State, SurfaceHarness, SystemTopic, manifest_of,
        operation::{Action, ErrorCode, Operation, PresentationAction, Refusal},
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

#[tokio::test]
async fn empty_and_unavailable_catalogues_are_distinct() {
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
        let drawn = Fixture::panel(&state).await.unwrap().draw();

        assert!(drawn.text().contains(text));
        assert!(drawn.node("results").unwrap().children.is_empty());
    }

    assert!(!Drawn::of::<Indicator>(&State::new()).unwrap().is_empty());
}

#[tokio::test]
async fn search_matches_all_tokens_and_ranks_names_before_metadata() {
    let mut panel = Fixture::panel(&Fixture::state()).await.unwrap();

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
    assert!(panel.draw().text().contains("No matching results"));

    let mut catalogue = Fixture::catalogue();
    catalogue.applications[0].keywords.push("terminal".into());
    panel.state(&State::new().with(catalogue));
    Fixture::edit(&mut panel, "terminal");

    assert_eq!(
        Inspect::ids(&mut panel),
        vec!["org.example.Terminal.desktop", "org.example.Files.desktop"]
    );
}

#[tokio::test]
async fn ordering_and_limits_are_stable_with_duplicate_labels() {
    let mut catalogue = Fixture::catalogue();
    catalogue.applications[0].name = "Same".into();
    catalogue.applications[1].name = "Same".into();
    let mut panel = Fixture::panel(&State::new().with(catalogue.clone()))
        .await
        .unwrap();
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
    let mut limited = Fixture::configured(&Fixture::state(), &settings)
        .await
        .unwrap();

    assert_eq!(Inspect::ids(&mut limited).len(), 1);
    assert!(limited.draw().text().contains("Showing 1 of 8"));
}

#[tokio::test]
async fn query_selection_and_clears_are_instance_local_and_revision_aware() {
    let mut first = Fixture::panel(&Fixture::state()).await.unwrap();
    let mut second = Fixture::panel(&Fixture::state()).await.unwrap();
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

    assert_eq!(first.model().selected.as_ref(), Some(&Inspect::id().into()));
    assert!(second.model().selected.is_none());
}

#[tokio::test]
async fn field_navigation_and_escape_bindings_are_declared() {
    let mut panel = Fixture::panel(&Fixture::state()).await.unwrap();
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
    let mut panel = Fixture::panel(&Fixture::state()).await.unwrap();
    Inspect::activate(&mut panel);
    panel.send(Message::Activate(Inspect::id().into())).unwrap();

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
    let mut panel = Fixture::panel(&Fixture::state()).await.unwrap();
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
    let mut panel = Fixture::panel(&Fixture::state()).await.unwrap();
    Fixture::edit(&mut panel, "terminal");
    panel.send(Message::Activate(Inspect::id().into())).unwrap();

    assert!(panel.take_effect().is_none());

    panel.send(Message::Clear).unwrap();
    let mut catalogue = Fixture::catalogue();
    catalogue.applications.remove(0);
    panel.state(&State::new().with(catalogue));
    panel.send(Message::Activate(Inspect::id().into())).unwrap();

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
    let mut panel = Fixture::panel(&Fixture::state()).await.unwrap();
    Inspect::activate(&mut panel);
    panel.lifecycle(Lifecycle::Hidden).unwrap();
    panel.lifecycle(Lifecycle::Presented).unwrap();

    assert!(panel.model().pending);

    panel.complete_effect(Ok(None)).await.unwrap();
    panel.complete().await.unwrap();

    assert!(!panel.model().pending);
    panel
        .take_effect()
        .unwrap()
        .commands()
        .unwrap()
        .complete()
        .unwrap();
    panel.complete().await.unwrap();
    assert!(panel.take_effect().is_none());
}

#[tokio::test]
async fn escape_hides_only_this_instance_and_reports_a_refusal() {
    let mut panel = Fixture::panel(&Fixture::state()).await.unwrap();
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

#[tokio::test]
async fn hide_and_close_clear_the_query_and_selection() {
    let mut panel = Fixture::panel(&Fixture::state()).await.unwrap();

    for lifecycle in [Lifecycle::Hidden, Lifecycle::Closed] {
        Fixture::edit(&mut panel, "files");
        panel.send(Message::Selected(Inspect::id().into())).unwrap();
        panel.lifecycle(lifecycle).unwrap();

        assert!(panel.model().query.text().is_empty());
        assert!(panel.model().selected.is_none());
    }

    assert_eq!(
        manifest_of(&plugin()).granted().unwrap(),
        vec![
            omega::internal::Capability::StateRead,
            omega::internal::Capability::Spawn
        ]
    );
}

#[tokio::test]
async fn fuzzy_search_handles_subsequences_and_preserves_literal_priority() {
    let mut panel = Fixture::panel(&Fixture::state()).await.unwrap();
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

#[tokio::test]
async fn favorites_sort_alphabetically_before_other_apps_without_changing_search_relevance() {
    let mut panel = Fixture::panel(&Fixture::state()).await.unwrap();
    Fixture::favorites(
        &mut panel,
        &[
            "org.example.Terminal.desktop",
            "org.example.Files.desktop",
            "removed.desktop",
        ],
    )
    .unwrap();
    assert_eq!(
        &Inspect::ids(&mut panel)[..3],
        &[
            "org.example.Files.desktop",
            "org.example.Terminal.desktop",
            "org.example.Browser.desktop",
        ]
    );
    assert_eq!(Inspect::ids(&mut panel).len(), 8);

    let mut catalogue = Fixture::catalogue();
    catalogue.applications[0].keywords.push("terminal".into());
    panel.state(&State::new().with(catalogue));
    Fixture::edit(&mut panel, "terminal");
    assert_eq!(
        Inspect::ids(&mut panel),
        vec!["org.example.Terminal.desktop", "org.example.Files.desktop",]
    );
}

#[tokio::test]
async fn favorite_button_inserts_only_the_selected_app_without_launching_or_dismissing() {
    let mut panel = Fixture::panel(&Fixture::state()).await.unwrap();
    Fixture::edit(&mut panel, "files");
    let drawn = panel.draw();
    panel.interact(&drawn, "favorite", "press", ()).unwrap();
    assert!(panel.model().saving);
    let insert = panel.expect_storage_insert::<Favorites>().await.unwrap();
    assert_eq!(insert.key(), &Inspect::id());
    assert_eq!(insert.value(), &());
    insert.succeed(Fixture::revision(5)).unwrap();
    panel.complete().await.unwrap();
    assert!(!panel.model().saving);
    assert!(panel.take_effect().is_none());
}

#[tokio::test]
async fn favorite_save_failure_is_visible_without_retrying() {
    let mut panel = Fixture::panel(&Fixture::state()).await.unwrap();
    Fixture::edit(&mut panel, "files");
    panel.send(Message::ToggleFavorite).unwrap();
    panel
        .expect_storage_insert::<Favorites>()
        .await
        .unwrap()
        .refuse(Refusal::unavailable("save refused"))
        .unwrap();
    panel.complete().await.unwrap();
    assert!(!panel.model().saving);
    assert!(panel.draw().text().contains("Could not save favorite"));
    assert!(panel.take_effect().is_none());
}

#[tokio::test]
async fn search_keeps_results_height_and_footer_nodes_stable() {
    let mut panel = Fixture::panel(&Fixture::state()).await.unwrap();
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

#[tokio::test]
async fn loading_favorites_disables_changes_but_leaves_search_available() {
    let mut panel = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();
    assert_eq!(panel.draw().flag("favorite", "disabled"), Some(true));
    assert!(panel.draw().text().contains("Loading favorites"));
    Fixture::edit(&mut panel, "files");
    assert_eq!(Inspect::ids(&mut panel), vec![Inspect::id().to_string()]);
    panel
        .take_effect()
        .unwrap()
        .complete(Err(EffectError::Refused(Refusal::unavailable(
            "storage offline",
        ))))
        .unwrap();
    assert!(panel.draw().text().contains("Favorites unavailable"));
    assert_eq!(panel.draw().flag("favorite", "disabled"), Some(true));
}

#[tokio::test]
async fn favorite_removal_uses_the_entry_revision_and_reports_conflicts_without_retry() {
    let mut panel = Fixture::panel(&Fixture::state()).await.unwrap();
    Fixture::favorites(&mut panel, &["org.example.Files.desktop"]).unwrap();
    panel.send(Message::ToggleFavorite).unwrap();
    let read = panel.expect_storage_read::<Favorites>().await.unwrap();
    assert_eq!(read.query().key(), Some(&Inspect::id()));
    read.reply(&Fixture::snapshot(&["org.example.Files.desktop"]))
        .unwrap();
    let remove = panel.expect_storage_remove::<Favorites>().await.unwrap();
    assert_eq!(remove.key(), &Inspect::id());
    assert_eq!(remove.expected(), &Fixture::revision(4));
    remove
        .refuse(Refusal::new(ErrorCode::Conflict, "changed concurrently"))
        .unwrap();
    panel.complete().await.unwrap();
    assert!(!panel.model().saving);
    assert!(panel.draw().text().contains("changed concurrently"));
    assert!(panel.take_effect().is_none());
}

#[tokio::test]
async fn launch_does_not_depend_on_favorites_storage() {
    let mut panel = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();
    panel
        .take_effect()
        .unwrap()
        .complete(Err(EffectError::Refused(Refusal::unavailable(
            "storage offline",
        ))))
        .unwrap();
    panel
        .take_effect()
        .unwrap()
        .commands()
        .unwrap()
        .complete()
        .unwrap();
    panel.complete().await.unwrap();
    Inspect::activate(&mut panel);
    assert!(matches!(
        panel.complete_effect(Ok(None)).await.unwrap(),
        Operation::Act(_)
    ));
    panel.complete().await.unwrap();
    assert!(matches!(
        panel.complete_effect(Ok(None)).await.unwrap(),
        Operation::ChangePresentation(_)
    ));
    panel.complete().await.unwrap();
    assert!(panel.take_effect().is_none());
}

#[tokio::test]
async fn snapshots_update_each_instance_without_resetting_its_query() {
    let mut bar = Fixture::panel(&Fixture::state()).await.unwrap();
    let mut overlay = Fixture::panel(&Fixture::state()).await.unwrap();
    Fixture::edit(&mut overlay, "files");
    Fixture::favorites(&mut bar, &["org.example.Terminal.desktop"]).unwrap();
    Fixture::favorites(&mut overlay, &["org.example.Terminal.desktop"]).unwrap();
    assert_eq!(Inspect::ids(&mut bar)[0], "org.example.Terminal.desktop");
    assert_eq!(overlay.model().query.text(), "files");
    assert_eq!(Inspect::ids(&mut overlay), vec![Inspect::id().to_string()]);
}

#[tokio::test]
async fn removing_a_favorite_preserves_other_favorites_and_does_not_dismiss() {
    let mut panel = Fixture::panel(&Fixture::state()).await.unwrap();
    Fixture::favorites(
        &mut panel,
        &["org.example.Files.desktop", "org.example.Terminal.desktop"],
    )
    .unwrap();
    panel.send(Message::ToggleFavorite).unwrap();
    panel
        .expect_storage_read::<Favorites>()
        .await
        .unwrap()
        .reply(&Fixture::snapshot(&["org.example.Files.desktop"]))
        .unwrap();
    let remove = panel.expect_storage_remove::<Favorites>().await.unwrap();
    assert_eq!(remove.key(), &Inspect::id());
    remove.succeed(Fixture::revision(6)).unwrap();
    panel.complete().await.unwrap();
    Fixture::favorites(&mut panel, &["org.example.Terminal.desktop"]).unwrap();
    assert_eq!(Inspect::ids(&mut panel)[0], "org.example.Terminal.desktop");
    assert_eq!(
        panel.draw().prop("favorite", "label").as_deref(),
        Some("Add favorite")
    );
    assert!(!panel.model().saving);
    assert!(panel.take_effect().is_none());
}

#[tokio::test]
async fn removing_an_already_absent_favorite_finishes_without_a_write() {
    let mut panel = Fixture::panel(&Fixture::state()).await.unwrap();
    Fixture::favorites(&mut panel, &["org.example.Files.desktop"]).unwrap();
    panel.send(Message::ToggleFavorite).unwrap();
    panel
        .expect_storage_read::<Favorites>()
        .await
        .unwrap()
        .reply(&Fixture::snapshot(&[]))
        .unwrap();
    panel.complete().await.unwrap();
    assert!(!panel.model().saving);
    assert!(panel.model().error.is_empty());
    assert!(panel.take_effect().is_none());
}

#[tokio::test]
async fn discovered_actions_use_typed_calls_and_never_write_favorites() {
    let mut panel = Fixture::panel(&Fixture::state()).await.unwrap();
    panel.lifecycle(Lifecycle::Presented).unwrap();
    panel
        .take_effect()
        .unwrap()
        .commands()
        .unwrap()
        .entry(audio::SetAudible, true)
        .entry(audio::SetVolume, false)
        .complete()
        .unwrap();
    panel.complete().await.unwrap();

    Fixture::edit(&mut panel, "mute sound");
    let mute = CandidateId::Action("audio/audible/false".into());
    assert!(Inspect::ids(&mut panel).contains(&mute.to_string()));
    assert!(
        !Inspect::ids(&mut panel)
            .iter()
            .any(|id| id.contains("volume"))
    );
    panel.send(Message::Selected(mute.clone())).unwrap();
    panel.send(Message::ToggleFavorite).unwrap();
    assert!(panel.take_effect().is_none());

    panel.send(Message::Activate(mute.clone())).unwrap();
    panel.send(Message::Activate(mute)).unwrap();
    let call = panel
        .take_effect()
        .unwrap()
        .command::<audio::SetAudible>()
        .unwrap();
    assert!(!*call.input());
    assert!(panel.take_effect().is_none());
    call.complete(Ok(())).unwrap();
    panel.complete().await.unwrap();
    assert!(matches!(
        panel.take_effect().unwrap().operation(),
        Operation::ChangePresentation(_)
    ));
}

#[tokio::test]
async fn semantic_ranking_preserves_selection_and_ignores_stale_or_invented_results() {
    let mut panel = Fixture::panel(&Fixture::state()).await.unwrap();
    Fixture::edit(&mut panel, "file");
    let id = Inspect::id();
    panel.send(Message::Selected(id.clone().into())).unwrap();
    let epoch = panel.model().epoch;
    panel
        .send(Message::Ranked {
            epoch,
            query: "file".into(),
            result: Ok(typesafe::Ranking {
                model: "fixture".into(),
                scores: [(id.to_string(), 1.9), ("invented".into(), 2.0)].into(),
            }),
        })
        .unwrap();
    assert_eq!(Inspect::ids(&mut panel), vec![id.to_string()]);
    assert_eq!(panel.model().selected, Some(id.into()));
    assert!(
        panel.take_effect().is_none(),
        "ranking does not execute a result"
    );

    Fixture::edit(&mut panel, "terminal");
    panel
        .send(Message::Ranked {
            epoch,
            query: "file".into(),
            result: Err(omega::Error::invalid("old failure")),
        })
        .unwrap();
    assert!(panel.model().error.is_empty());
    assert!(panel.model().semantic.is_none());
    assert_eq!(
        Inspect::ids(&mut panel),
        vec!["org.example.Terminal.desktop"]
    );
}

#[tokio::test]
async fn discovery_failure_keeps_application_search_available() {
    let mut panel = Fixture::panel(&Fixture::state()).await.unwrap();
    panel
        .send(Message::CommandsLoaded {
            epoch: panel.model().epoch,
            result: Err(omega::Error::invalid("offline")),
        })
        .unwrap();
    assert!(panel.draw().text().contains("Commands unavailable"));
    Fixture::edit(&mut panel, "files");
    assert_eq!(Inspect::ids(&mut panel), vec![Inspect::id().to_string()]);
    assert!(
        panel.draw().node("semantic-search").is_none(),
        "remote evaluation is opt-in"
    );
}
