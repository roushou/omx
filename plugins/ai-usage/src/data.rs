use serde::Deserialize;
use std::collections::BTreeMap;

/// Validated collector identity; also used to retain panel selection.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, omega::Config)]
pub struct ProviderId {
    pub value: String,
}

impl ProviderId {
    pub fn parse(value: &str) -> Result<Self, String> {
        if value.is_empty()
            || value.len() > 64
            || !value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err("invalid provider identity".into());
        }
        Ok(Self {
            value: value.into(),
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, omega::PluginState)]
pub struct Snapshot {
    pub providers: Vec<Provider>,
    pub refreshing: bool,
    pub loaded: bool,
    pub error: String,
    pub now: i64,
}

#[derive(Debug, Clone, Default, PartialEq, omega::Config)]
pub struct Provider {
    pub id: ProviderId,
    pub name: String,
    pub plan: String,
    pub ready: bool,
    pub has_stats: bool,
    pub has_prompt_stats: bool,
    pub scope: String,
    pub updated: i64,
    pub status: String,
    pub auth_help: String,
    pub limits: Vec<Limit>,
    pub balance: Option<Balance>,
    pub today_tokens: u64,
    pub today_prompts: u64,
    pub today_sessions: u64,
    pub total_prompts: u64,
    pub total_sessions: u64,
    pub days: Vec<Day>,
    pub models: Vec<ModelUsage>,
}

#[derive(Debug, Clone, Default, PartialEq, omega::Config, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Limit {
    pub label: String,
    pub percent: f64,
    pub resets_at: String,
}

#[derive(Debug, Clone, Default, PartialEq, omega::Config, Deserialize)]
#[serde(default)]
pub struct Balance {
    pub remaining: f64,
    pub funded: f64,
    pub spent: f64,
    pub currency: String,
    pub estimated: bool,
}

#[derive(Debug, Clone, Default, PartialEq, omega::Config, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Day {
    pub date: String,
    pub message_count: u64,
}

#[derive(Debug, Clone, Default, PartialEq, omega::Config)]
pub struct ModelUsage {
    pub name: String,
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
}

impl ModelUsage {
    pub fn total(&self) -> u64 {
        self.input
            .saturating_add(self.output)
            .saturating_add(self.cache_read)
            .saturating_add(self.cache_write)
    }
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Record {
    schema_version: u32,
    id: String,
    name: String,
    updated_at: String,
    tier_label: String,
    ready: bool,
    has_local_stats: bool,
    has_prompt_stats: Option<bool>,
    scope: String,
    usage_status_text: String,
    auth_help_text: String,
    limits: Vec<Limit>,
    balance: Option<Balance>,
    today_total_tokens: u64,
    today_prompts: u64,
    today_sessions: u64,
    total_prompts: u64,
    total_sessions: u64,
    recent_days: Vec<Day>,
    model_usage: BTreeMap<String, Bucket>,
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Bucket {
    input_tokens: u64,
    output_tokens: u64,
    cache_read_input_tokens: u64,
    cache_creation_input_tokens: u64,
}

impl Provider {
    pub fn parse(bytes: &[u8], expected: &ProviderId) -> Result<Self, String> {
        let record: Record = serde_json::from_slice(bytes).map_err(|_| "invalid usage JSON")?;

        if record.schema_version != 1 {
            return Err("unsupported usage schema".into());
        }

        let id = ProviderId::parse(&record.id)?;

        if &id != expected {
            return Err("provider identity does not match filename".into());
        }

        if record.limits.len() > 8
            || record.recent_days.len() > 31
            || record.model_usage.len() > 128
        {
            return Err("usage record exceeds display limits".into());
        }

        for limit in &record.limits {
            if !limit.percent.is_finite() || limit.percent < 0.0 {
                return Err("invalid usage percentage".into());
            }

            if !limit.resets_at.is_empty()
                && chrono::DateTime::parse_from_rfc3339(&limit.resets_at).is_err()
            {
                return Err("invalid reset timestamp".into());
            }
        }

        if let Some(balance) = &record.balance
            && [balance.remaining, balance.funded, balance.spent]
                .iter()
                .any(|v| !v.is_finite() || *v < 0.0)
        {
            return Err("invalid credit balance".into());
        }

        let updated = chrono::DateTime::parse_from_rfc3339(&record.updated_at)
            .map_err(|_| "invalid update timestamp")?
            .timestamp();
        let mut models = Vec::new();

        for (name, bucket) in record.model_usage {
            if name.trim().is_empty() || name.len() > 256 || name.chars().any(char::is_control) {
                return Err("invalid model name".into());
            }

            models.push(ModelUsage {
                name,
                input: bucket.input_tokens,
                output: bucket.output_tokens,
                cache_read: bucket.cache_read_input_tokens,
                cache_write: bucket.cache_creation_input_tokens,
            });
        }

        models.sort_by(|a, b| b.total().cmp(&a.total()).then(a.name.cmp(&b.name)));
        let mut days = record.recent_days;

        for day in &days {
            chrono::NaiveDate::parse_from_str(&day.date, "%Y-%m-%d")
                .map_err(|_| "invalid usage day")?;
        }

        days.sort_by(|a, b| a.date.cmp(&b.date));

        if days.windows(2).any(|pair| pair[0].date == pair[1].date) {
            return Err("duplicate usage day".into());
        }

        if days.len() > 7 {
            days.drain(..days.len() - 7);
        }
        Ok(Self {
            id,
            name: if record.name.is_empty() {
                record.id
            } else {
                record.name
            },
            plan: record.tier_label,
            ready: record.ready,
            has_stats: record.has_local_stats,
            has_prompt_stats: record.has_prompt_stats.unwrap_or(true),
            scope: record.scope,
            updated,
            status: record.usage_status_text,
            auth_help: record.auth_help_text,
            limits: record.limits,
            balance: record.balance,
            today_tokens: record.today_total_tokens,
            today_prompts: record.today_prompts,
            today_sessions: record.today_sessions,
            total_prompts: record.total_prompts,
            total_sessions: record.total_sessions,
            days,
            models,
        })
    }

    pub fn stale(&self, now: i64) -> bool {
        now.saturating_sub(self.updated) > 30 * 60
    }
}
