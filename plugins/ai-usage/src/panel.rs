use crate::{
    Refresh, Settings,
    data::{Provider, ProviderId, Snapshot},
};
mod sections;

use desktop_ui::PanelHeader;
use omega::{
    Surface, View,
    record::Watch,
    surface::{Events, Task},
    ui::{Button, Choice, Column, Component, Glyph, Icon, Size, Text},
};
use sections::ProviderSections;

#[derive(Debug, omega::Surface)]
pub struct Panel {
    snapshot: Watch<Snapshot>,
    #[omega(config)]
    settings: Settings,
}

#[derive(Default)]
pub struct Model {
    selected: Option<ProviderId>,
}

pub enum Message {
    Select(String),
}

impl Surface for Panel {
    type Model = Model;
    type Message = Message;
    type Effects = ();

    fn update(&self, model: &mut Model, message: Message, _: &()) -> Task<Message> {
        match message {
            Message::Select(value) => {
                if let Ok(id) = ProviderId::parse(&value)
                    && self
                        .settings
                        .visible(&self.snapshot.get())
                        .iter()
                        .any(|p| p.id == id)
                {
                    model.selected = Some(id);
                }
            }
        }
        Task::none()
    }

    fn render(&self, model: &Model, events: &Events<Message>) -> View {
        let snapshot = self.snapshot.get();
        let providers = self.settings.visible(&snapshot);
        let selected = model
            .selected
            .as_ref()
            .and_then(|id| providers.iter().find(|p| &p.id == id).copied())
            .or_else(|| {
                providers
                    .iter()
                    .find(|p| p.id.value == self.settings.preferred)
                    .copied()
            })
            .or_else(|| providers.first().copied());
        let mut panel = Column::new().width(440).gap(20).child(
            PanelHeader::labelled(
                "AI usage",
                if snapshot.refreshing {
                    "Refreshing"
                } else {
                    "Subscriptions"
                },
            )
            .leading(Icon::new(Glyph::Cpu).size(Size::Display))
            .trailing(
                Button::new("Refresh")
                    .key("refresh")
                    .disabled_if(snapshot.refreshing)
                    .on_press(Refresh),
            )
            .render(),
        );

        if !snapshot.error.is_empty() {
            panel = panel.child(Text::new(&snapshot.error).warning().key("collection-error"));
        }

        let Some(provider) = selected else {
            return panel
                .child(
                    Text::new(if snapshot.refreshing || !snapshot.loaded {
                        "Loading usage…"
                    } else {
                        "No usage records found. Sign into an AI coding tool, use it, then refresh."
                    })
                    .key("empty"),
                )
                .into();
        };

        if providers.len() > 1 {
            panel = panel.child(
                Choice::new()
                    .key("providers")
                    .options(
                        providers
                            .iter()
                            .map(|p| (p.id.value.clone(), Text::new(&p.name))),
                    )
                    .selected(Some(provider.id.value.clone()))
                    .on_select(events.on(Message::Select)),
            );
        }
        let sections = ProviderSections {
            provider,
            now: snapshot.now,
        };

        panel
            .children(sections.summary())
            .children(sections.allowances())
            .children(sections.balance())
            .child(sections.today())
            .children(sections.days())
            .children(sections.models())
            .children(sections.recorded_counts())
            .into()
    }
}

pub(crate) struct Format;
impl Format {
    pub(crate) fn tokens(value: u64) -> String {
        if value >= 1_000_000_000 {
            format!("{:.1}B", value as f64 / 1_000_000_000.0)
        } else if value >= 1_000_000 {
            format!("{:.1}M", value as f64 / 1_000_000.0)
        } else if value >= 1_000 {
            format!("{:.1}k", value as f64 / 1_000.0)
        } else {
            value.to_string()
        }
    }

    pub(crate) fn reset(value: &str, now: i64) -> String {
        let Ok(time) = chrono::DateTime::parse_from_rfc3339(value) else {
            return "Reset time unavailable".into();
        };

        let minutes = (time.timestamp().saturating_sub(now).max(0) + 59) / 60;

        if minutes == 0 {
            "Reset due · refresh for the latest allowance".into()
        } else if minutes >= 1440 {
            format!("Resets in {}d {}h", minutes / 1440, minutes % 1440 / 60)
        } else if minutes >= 60 {
            format!("Resets in {}h {}m", minutes / 60, minutes % 60)
        } else {
            format!("Resets in {minutes}m")
        }
    }
    fn updated(provider: &Provider, now: i64) -> String {
        let minutes = now.saturating_sub(provider.updated).max(0) / 60;
        format!(
            "{}Updated {}m ago",
            if provider.stale(now) { "Stale · " } else { "" },
            minutes
        )
    }
}
