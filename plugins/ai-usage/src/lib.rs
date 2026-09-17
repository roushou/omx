//! AI subscription usage collected by Omarchy and shared across all plugin instances.
mod data;
mod panel;
mod source;

pub use data::Snapshot;
use omega::{
    Command, Surface, View,
    record::{Own, Watch},
    surface::{Events, Task},
    ui::{Glyph, Icon, Row, Text},
};
pub use panel::{Message, Model, Panel};
use std::convert::Infallible;

/// Presentation preferences. An empty provider list shows providers with available limits or recorded usage.
#[derive(Debug, Clone, omega::Config)]
pub struct Settings {
    pub providers: Vec<String>,
    pub preferred: String,
    pub show_label: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            providers: Vec::new(),
            preferred: "codex".into(),
            show_label: true,
        }
    }
}

impl Settings {
    fn visible<'a>(&self, snapshot: &'a Snapshot) -> Vec<&'a data::Provider> {
        snapshot
            .providers
            .iter()
            .filter(|p| {
                if self.providers.is_empty() {
                    p.ready || p.has_stats || !p.limits.is_empty() || p.balance.is_some()
                } else {
                    self.providers.contains(&p.id.value)
                }
            })
            .collect()
    }
}

/// Bar summary. The panel and every monitor read the same collected snapshot.
#[derive(Debug, omega::Surface)]
pub struct Indicator {
    snapshot: Watch<Snapshot>,
    #[omega(config)]
    settings: Settings,
}

impl Surface for Indicator {
    type Model = ();
    type Message = Infallible;
    type Effects = ();

    fn update(&self, _: &mut (), message: Infallible, _: &()) -> Task<Infallible> {
        match message {}
    }

    fn render(&self, _: &(), _: &Events<Infallible>) -> View {
        let snapshot = self.snapshot.get();
        let providers = self.settings.visible(&snapshot);
        let tooltip = if providers.is_empty() {
            if snapshot.refreshing {
                "AI usage · Refreshing".into()
            } else {
                "AI usage · Open for status and setup".into()
            }
        } else {
            providers
                .iter()
                .map(|p| {
                    format!(
                        "{} · {}{}",
                        p.name,
                        if p.has_stats {
                            format!("{} tokens today", panel::Format::tokens(p.today_tokens))
                        } else {
                            "Token usage unavailable".into()
                        },
                        if p.stale(snapshot.now) {
                            " · Stale"
                        } else {
                            ""
                        }
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        let mut row = Row::new().gap(6).tooltip(tooltip).child(Icon::named("󱚣"));

        if self.settings.show_label {
            row = row.child(Text::new("AI"));
        }

        if !snapshot.error.is_empty() {
            row = row.child(Icon::new(Glyph::Warning));
        }

        row.into()
    }
}

/// Admit an immediate collection. Repeated requests join the active collection.
/// Completion acknowledges admission; the shared snapshot reports progress and errors.
#[derive(Debug, omega::Command)]
#[omega(name = "refresh")]
pub struct Refresh {
    snapshot: Own<Snapshot>,
}

impl Command for Refresh {
    type Input = ();
    type Output = ();

    async fn call(&self, _: ()) -> omega::Result<()> {
        self.snapshot
            .set(&source::Source::shared().poll(true))
            .await
    }
}

/// Publish worker progress and refresh automatically when due. Schedule every 5 seconds.
/// Collection runs every 15 minutes, or after 1 minute following a collection failure.
#[derive(Debug, omega::Command)]
#[omega(name = "poll")]
pub struct Poll {
    snapshot: Own<Snapshot>,
}

impl Command for Poll {
    type Input = ();
    type Output = ();

    async fn call(&self, _: ()) -> omega::Result<()> {
        let next = source::Source::shared().poll(false);

        if self.snapshot.get() != next {
            self.snapshot.set(&next).await?;
        }
        Ok(())
    }
}

pub fn plugin() -> omega::Plugin {
    omega::plugin!()
        .surface(Indicator)
        .surface(Panel)
        .command::<Refresh>()
        .command::<Poll>()
}

#[cfg(test)]
mod previews;
#[cfg(test)]
mod tests;
