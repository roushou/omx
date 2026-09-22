//! Rust is the source of truth for the shell layout and plugin instances.

use std::time::Duration;

use omega_document::{Actions, Cadence, Document, Host, Schedules};
use omega_omarchy::shell::{Bar, Idle, Native, PluginWidget, Shell};

fn main() -> omega_document::Result<()> {
    System::document()?.emit()?;
    Ok(())
}

struct System;

impl System {
    fn document() -> omega_document::Result<Document> {
        Ok(Document::new()
            .command_host(audio_commands::Host::declaration())?
            .command_host(bluetooth_commands::Host::declaration())?
            .command_host(display_commands::Host::declaration())?
            .command_host(media_commands::Host::declaration())?
            .command_host(power_commands::Host::declaration())?
            .command_host(workspaces_commands::Host::declaration())?
            .command_host(ai_usage_commands::Host::declaration())?
            .command_host(capture_commands::Host::declaration())?
            .env("OMEGA_HOST", Host::name())
            .schedule(Schedules::every(
                "ai-usage-poll",
                Cadence::seconds(5),
                Actions::invoke(ai_usage_commands::Poll),
            ))
            .with(
                Shell::new()
                    .idle(
                        Idle::new()
                            .screensaver_after(Duration::from_secs(1500))
                            .lock_after(Duration::from_secs(3000)),
                    )
                    .bar(
                        Bar::top()
                            .left([
                                Native::menu().into(),
                                PluginWidget::new("launcher", launcher::Indicator)
                                    .panel(launcher::Panel)
                                    .into(),
                                PluginWidget::new("workspaces", workspaces::Indicator).into(),
                                PluginWidget::new("media", media::Indicator)
                                    .panel(media::Panel)
                                    .into(),
                            ])
                            .center_anchor("omega.view")
                            .center([PluginWidget::new("clock", clock::Indicator)
                                .panel(clock::Panel)
                                .into()])
                            .right([
                                Native::tray().into(),
                                PluginWidget::new("ai-usage", ai_usage::Indicator)
                                    .panel(ai_usage::Panel)
                                    .into(),
                                PluginWidget::new("system-monitor", system_monitor::Indicator)
                                    .panel(system_monitor::Panel)
                                    .into(),
                                PluginWidget::new("bluetooth", bluetooth::Indicator)
                                    .panel(bluetooth::Panel)
                                    .into(),
                                PluginWidget::new("network", network::Indicator)
                                    .panel(network::Panel)
                                    .into(),
                                PluginWidget::new("audio", audio::Indicator)
                                    .panel(audio::Panel)
                                    .into(),
                                PluginWidget::new("display", display::Indicator)
                                    .panel(display::Panel)
                                    .into(),
                                PluginWidget::new("power", power::Indicator)
                                    .panel(power::Panel)
                                    .into(),
                            ]),
                    ),
            )?)
    }
}
