use super::Format;
use crate::data::Provider;
use desktop_ui::{Detail, LabelledControl, Section};
use omega::{
    Percent, View,
    ui::{Column, Component, Header, Progress, Row, Size, Text},
};

pub(super) struct ProviderSections<'a> {
    pub provider: &'a Provider,
    pub now: i64,
}

impl ProviderSections<'_> {
    pub fn summary(&self) -> Vec<View> {
        let provider = self.provider;
        let mut views = Vec::new();
        views.push(Header::new(&provider.name).key("provider-name").into());

        if !provider.plan.is_empty() {
            views.push(Text::new(&provider.plan).muted().into());
        }
        views.push(
            Text::new(Format::updated(provider, self.now))
                .size(Size::Body)
                .muted()
                .key("freshness")
                .into(),
        );

        if !provider.status.is_empty() {
            views.push(
                Text::new(&provider.status)
                    .warning()
                    .key("provider-status")
                    .into(),
            );

            if !provider.auth_help.is_empty() {
                views.push(Text::new(&provider.auth_help).muted().into());
            }
        }

        views
    }

    pub fn allowances(&self) -> Option<View> {
        let provider = self.provider;
        let mut view = None;
        if !provider.limits.is_empty() {
            let mut limits = Section::new()
                .heading(Header::new("Allowance used"))
                .gap(16);

            for (index, limit) in provider.limits.iter().enumerate() {
                limits = limits.child(
                    Column::new()
                        .gap(6)
                        .key(format!("limit-{index}"))
                        .child(Detail::row(
                            &limit.label,
                            format!("{:.0}%", limit.percent * 100.0),
                        ))
                        .child(Progress::new(Percent::of(limit.percent)))
                        .child(
                            Text::new(Format::reset(&limit.resets_at, self.now))
                                .size(Size::Body)
                                .muted(),
                        ),
                );
            }
            view = Some(limits.render());
        }

        view
    }

    pub fn balance(&self) -> Option<View> {
        let provider = self.provider;
        let mut view = None;
        if let Some(balance) = &provider.balance {
            let mut section = Section::new()
                .heading(Header::new(if balance.estimated {
                    "Estimated credit"
                } else {
                    "Credit balance"
                }))
                .child(Detail::row(
                    "Remaining",
                    format!("{} {:.2}", balance.currency, balance.remaining),
                ));

            if balance.funded > 0.0 {
                section = section
                    .child(Progress::new(Percent::of(
                        balance.remaining / balance.funded,
                    )))
                    .child(Detail::row(
                        "Funded / spent",
                        format!(
                            "{:.2} / {:.2} {}",
                            balance.funded, balance.spent, balance.currency
                        ),
                    ));
            }
            view = Some(section.render());
        }

        view
    }

    pub fn today(&self) -> View {
        let provider = self.provider;
        if provider.has_stats {
            let mut today = Row::new().gap(32).child(Detail::tile(
                "Tokens",
                Format::tokens(provider.today_tokens),
            ));

            if provider.has_prompt_stats {
                today = today
                    .child(Detail::tile("Prompts", provider.today_prompts))
                    .child(Detail::tile("Sessions", provider.today_sessions));
            }
            Section::new()
                .heading(Header::new(if provider.scope == "account" {
                    "Today · account usage"
                } else {
                    "Today"
                }))
                .child(today)
                .render()
        } else {
            Text::new("Token history unavailable")
                .muted()
                .key("stats-unavailable")
                .into()
        }
    }

    pub fn days(&self) -> Option<View> {
        let provider = self.provider;
        let mut view = None;
        if !provider.days.is_empty() {
            let peak = provider
                .days
                .iter()
                .map(|d| d.message_count)
                .max()
                .unwrap_or(1)
                .max(1) as f64;
            view = Some(
                Section::new()
                    .heading(Header::new("Last seven days"))
                    .gap(12)
                    .children(provider.days.iter().map(|day| {
                        LabelledControl::new(
                            Text::new(&day.date).muted(),
                            Text::new(Format::tokens(day.message_count)),
                            Progress::new(Percent::of(day.message_count as f64 / peak)),
                        )
                        .gap(6)
                        .render()
                        .key(format!("day-{}", day.date))
                    }))
                    .render(),
            );
        }

        view
    }

    pub fn models(&self) -> Option<View> {
        let provider = self.provider;
        let mut view = None;
        if !provider.models.is_empty() {
            let peak = provider.models[0].total().max(1) as f64;
            view = Some(
                Section::new()
                    .heading(Header::new("Top models · recorded history"))
                    .gap(12)
                    .children(provider.models.iter().take(8).map(|model| {
                        LabelledControl::new(
                            Text::new(&model.name).muted(),
                            Text::new(Format::tokens(model.total())),
                            Progress::new(Percent::of(model.total() as f64 / peak)),
                        )
                        .gap(6)
                        .render()
                        .key(format!("model-{}", model.name))
                        .tooltip(format!(
                            "Input {} · Output {} · Cache read {} · Cache write {}",
                            model.input, model.output, model.cache_read, model.cache_write
                        ))
                    }))
                    .render(),
            );
        }

        view
    }

    pub fn recorded_counts(&self) -> Option<View> {
        let provider = self.provider;
        let mut view = None;
        if provider.has_stats && provider.has_prompt_stats {
            view = Some(
                Detail::row(
                    "Recorded prompts / sessions",
                    format!("{} / {}", provider.total_prompts, provider.total_sessions),
                )
                .key("recorded-counts"),
            );
        }

        view
    }
}
