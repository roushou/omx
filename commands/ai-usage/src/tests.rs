use super::*;
use crate::{
    data::{Provider, ProviderId},
    source,
};
use omega::config::Fields;
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
}
#[test]
fn collector_records_preserve_meaning_and_roundtrip_shared_state() {
    let provider = Fixture::provider();

    assert_eq!(provider.limits[0].percent, 0.42);
    assert_eq!(provider.models[0].total(), 145000);
    assert_eq!(provider.days[1].message_count, 145000);
    let snapshot = Snapshot {
        providers: vec![provider],
        ..Default::default()
    };

    assert_eq!(Snapshot::read(&snapshot.write()), snapshot);
}

#[test]
fn invalid_identity_schema_and_percentages_are_rejected() {
    let id = ProviderId::parse("codex").unwrap();

    for (key, value) in [
        ("id", serde_json::json!("../codex")),
        ("id", serde_json::json!("claude")),
        ("schemaVersion", serde_json::json!(2)),
        ("updatedAt", serde_json::json!("yesterday")),
    ] {
        let mut record = Fixture::record();
        record[key] = value;

        assert!(Provider::parse(&serde_json::to_vec(&record).unwrap(), &id).is_err());
    }

    let mut record = Fixture::record();
    record["limits"][0]["percent"] = serde_json::json!(-0.5);

    assert!(Provider::parse(&serde_json::to_vec(&record).unwrap(), &id).is_err());
    record["limits"][0]["percent"] = serde_json::json!(1.2);

    assert_eq!(
        Provider::parse(&serde_json::to_vec(&record).unwrap(), &id)
            .unwrap()
            .limits[0]
            .percent,
        1.2
    );
}

#[test]
fn future_fields_are_accepted_but_duplicate_days_are_refused() {
    let mut record = Fixture::record();
    record["futureField"] = serde_json::json!(true);
    let id = ProviderId::parse("codex").unwrap();

    assert!(Provider::parse(&serde_json::to_vec(&record).unwrap(), &id).is_ok());
    record["recentDays"][1] = record["recentDays"][0].clone();

    assert!(Provider::parse(&serde_json::to_vec(&record).unwrap(), &id).is_err());
}

#[test]
fn one_bad_record_does_not_hide_other_providers() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("codex.json"),
        serde_json::to_vec(&Fixture::record()).unwrap(),
    )
    .unwrap();
    std::fs::write(dir.path().join("claude.json"), b"broken").unwrap();
    let (providers, errors) = source::Records::read(dir.path()).unwrap();

    assert_eq!(providers.len(), 1);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].1.contains("claude"));
    assert!(
        source::Records::read(&dir.path().join("missing"))
            .unwrap()
            .0
            .is_empty()
    );
}

#[test]
fn oversized_and_symlinked_records_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("codex.json"), vec![b' '; 512 * 1024 + 1]).unwrap();
    std::os::unix::fs::symlink(
        dir.path().join("codex.json"),
        dir.path().join("claude.json"),
    )
    .unwrap();
    let (providers, errors) = source::Records::read(dir.path()).unwrap();

    assert!(providers.is_empty());
    assert_eq!(errors.len(), 2);
}
