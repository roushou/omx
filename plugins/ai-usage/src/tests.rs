use super::*;
use data::{Provider, ProviderId};
use omega::{
    config::Fields,
    record::PluginState,
    testing::{Drawn, State, SurfaceHarness},
};

pub(crate) struct Fixture;
impl Fixture {
    pub(crate) fn record() -> serde_json::Value {
        serde_json::json!({"schemaVersion":1,"ready":true,"hasLocalStats":true,"id":"codex","name":"Codex","updatedAt":"2026-09-16T00:00:00Z","tierLabel":"Plus",
            "todayTotalTokens":145000,"todayPrompts":12,"todaySessions":3,"totalPrompts":200,"totalSessions":40,
            "limits":[{"label":"5h window","percent":0.42,"resetsAt":"2026-09-16T02:30:00Z"},{"label":"Weekly","percent":0.71,"resetsAt":"2026-09-19T00:00:00Z"}],
            "recentDays":[{"date":"2026-09-15","messageCount":430000},{"date":"2026-09-16","messageCount":145000}],
            "modelUsage":{"gpt-example":{"inputTokens":15000,"outputTokens":25000,"cacheReadInputTokens":100000,"cacheCreationInputTokens":5000}}})
    }

    pub(crate) fn provider() -> Provider {
        Provider::parse(
            &serde_json::to_vec(&Self::record()).unwrap(),
            &ProviderId::parse("codex").unwrap(),
        )
        .unwrap()
    }

    pub(crate) fn snapshot() -> Snapshot {
        let codex = Self::provider();
        let mut claude = codex.clone();
        claude.id = ProviderId::parse("claude").unwrap();
        claude.name = "Claude".into();
        claude.plan.clear();
        claude.limits.clear();
        claude.status = "Claude limits unavailable".into();
        claude.auth_help = "Sign into Claude Code, then refresh.".into();
        let mut fireworks = codex.clone();
        fireworks.id = ProviderId::parse("fireworks").unwrap();
        fireworks.name = "Fireworks".into();
        fireworks.plan.clear();
        fireworks.has_prompt_stats = false;
        fireworks.scope = "account".into();
        fireworks.limits.clear();
        fireworks.balance = Some(data::Balance {
            remaining: 12.50,
            funded: 20.0,
            spent: 7.50,
            currency: "USD".into(),
            estimated: true,
        });
        Snapshot {
            now: codex.updated,
            providers: vec![codex, claude, fireworks],
            loaded: true,
            ..Default::default()
        }
    }

    pub(crate) fn stale() -> Snapshot {
        let mut value = Self::snapshot();
        value.now += 3600;
        value.error = "Collector unavailable; showing cached data".into();
        value
    }

    pub(crate) fn state(snapshot: Snapshot) -> State {
        State::new().keyspace(&Snapshot::address(), snapshot.write())
    }
}

#[test]
fn panel_selection_is_local_and_survives_refreshes() {
    let mut first = SurfaceHarness::<Panel>::new(&Fixture::state(Fixture::snapshot())).unwrap();
    let mut second = SurfaceHarness::<Panel>::new(&Fixture::state(Fixture::snapshot())).unwrap();
    first.send(Message::Select("claude".into())).unwrap();

    assert_eq!(
        first.draw().prop("provider-name", "text").as_deref(),
        Some("Claude")
    );

    assert_eq!(
        second.draw().prop("provider-name", "text").as_deref(),
        Some("Codex")
    );

    first.state(&Fixture::state(Fixture::stale()));

    assert_eq!(
        first.draw().prop("provider-name", "text").as_deref(),
        Some("Claude")
    );

    let mut snapshot = Fixture::snapshot();
    snapshot.providers.retain(|p| p.id.value != "claude");
    first.state(&Fixture::state(snapshot));

    assert_eq!(
        first.draw().prop("provider-name", "text").as_deref(),
        Some("Codex")
    );
}

#[test]
fn pending_refresh_disables_the_action_and_stale_data_is_labelled() {
    let mut snapshot = Fixture::stale();
    snapshot.refreshing = true;
    let drawn = Drawn::of::<Panel>(&Fixture::state(snapshot)).unwrap();

    assert_eq!(drawn.flag("refresh", "disabled"), Some(true));
    assert!(
        drawn
            .prop("freshness", "text")
            .unwrap()
            .starts_with("Stale")
    );

    assert!(drawn.node("collection-error").is_some());
    assert_eq!(
        drawn.node("refresh").unwrap().events["press"].command,
        "ai-usage.refresh"
    );
}

#[test]
fn empty_and_loading_panels_have_a_path_to_refresh() {
    for snapshot in [
        Snapshot::default(),
        Snapshot {
            loaded: true,
            ..Default::default()
        },
    ] {
        let drawn = Drawn::of::<Panel>(&Fixture::state(snapshot)).unwrap();

        assert!(drawn.node("empty").is_some());
        assert!(drawn.node("refresh").is_some());
        assert!(drawn.node("providers").is_none());
    }
}

#[test]
fn time_and_token_formatting_do_not_invent_allowances() {
    let now = Fixture::provider().updated;

    assert_eq!(
        panel::Format::reset("2026-09-16T02:30:00Z", now),
        "Resets in 2h 30m"
    );

    assert!(panel::Format::reset("2026-09-15T00:00:00Z", now).starts_with("Reset due"));
    assert_eq!(panel::Format::reset("", now), "Reset time unavailable");
    assert_eq!(panel::Format::tokens(12500), "12.5k");
}

#[test]
fn missing_stats_are_not_displayed_as_zero_activity() {
    let mut snapshot = Fixture::snapshot();
    snapshot.providers[0].has_stats = false;
    let drawn = Drawn::of::<Panel>(&Fixture::state(snapshot)).unwrap();

    assert!(drawn.node("stats-unavailable").is_some());
    assert!(drawn.node("recorded-counts").is_none());
    let mut panel = SurfaceHarness::<Panel>::new(&Fixture::state(Fixture::snapshot())).unwrap();
    panel.send(Message::Select("fireworks".into())).unwrap();

    assert!(panel.draw().node("recorded-counts").is_none());
}

#[test]
fn unconfigured_placeholder_is_hidden_unless_requested() {
    let mut snapshot = Fixture::snapshot();
    let provider = &mut snapshot.providers[2];
    provider.ready = false;
    provider.has_stats = false;
    provider.balance = None;

    assert_eq!(Settings::default().visible(&snapshot).len(), 2);
    let settings = Settings {
        providers: vec!["fireworks".into()],
        ..Default::default()
    };

    assert_eq!(settings.visible(&snapshot).len(), 1);
}
