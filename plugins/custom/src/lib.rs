//! A bar widget that shows a shell command's output, refreshed on an interval.
//!
//! This is the configurable "command module": the daemon runs the command
//! (bounded, with a timeout) and the widget renders its trimmed stdout. Each
//! placement supplies its own settings, so the same plugin serves several
//! different commands.
//!
//! ```text
//! PluginWidget::new("kernel", custom::Custom)
//!     .settings(&custom::Settings { command: "uname -r".into(), interval_seconds: 3600 })
//! ```

use omega::platform::process::Shell;
use omega::surface::{Events, Task};
use omega::ui::Text;
use omega::{Surface, View};
use std::time::Duration;

/// What to run and how often.
#[derive(Debug, Clone, omega::Config)]
pub struct Settings {
    /// The shell command; its trimmed stdout is the displayed text.
    pub command: String,
    /// Seconds between runs. Zero is treated as one.
    pub interval_seconds: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            command: "date +%H:%M".into(),
            interval_seconds: 60,
        }
    }
}

/// The last captured text, and the last failure if the command did not answer.
#[derive(Debug, Default)]
pub struct Model {
    text: String,
    error: String,
}

/// Capture the command's output on an interval.
#[derive(Debug, omega::Surface)]
pub struct Custom {
    #[omega(config)]
    settings: Settings,
}

/// The capture effect.
#[derive(Debug, omega::Effects)]
pub struct Effects {
    shell: Shell,
}

/// A captured result, and the next interval elapsing.
pub enum Message {
    Refreshed(Result<String, omega::Error>),
    Tick,
}

impl Surface for Custom {
    type Model = Model;
    type Message = Message;
    type Effects = Effects;

    fn mounted(&self, _model: &mut Model, effects: &Self::Effects) -> Task<Message> {
        Self::capture(&self.settings, effects)
    }

    fn update(
        &self,
        model: &mut Model,
        message: Message,
        effects: &Self::Effects,
    ) -> Task<Message> {
        match message {
            Message::Refreshed(Ok(text)) => {
                model.text = text;
                model.error.clear();
            }
            Message::Refreshed(Err(error)) => model.error = error.to_string(),
            Message::Tick => return Self::capture(&self.settings, effects),
        }
        Self::wait(self.settings.interval_seconds)
    }

    fn render(&self, model: &Model, _: &Events<Message>) -> View {
        if model.text.is_empty() && !model.error.is_empty() {
            // A command that cannot answer is not an empty reading.
            return Text::new("\u{2014}").muted().error().into();
        }
        Text::new(&model.text).into()
    }
}

impl Custom {
    fn capture(settings: &Settings, effects: &Effects) -> Task<Message> {
        Task::perform(
            effects.shell.capture(settings.command.clone()),
            Message::Refreshed,
        )
    }

    /// Wait one interval, then tick.
    fn wait(interval_seconds: u64) -> Task<Message> {
        let seconds = interval_seconds.max(1);
        Task::perform(
            async move {
                tokio::time::sleep(Duration::from_secs(seconds)).await;
                Ok(())
            },
            |_| Message::Tick,
        )
    }
}

/// Register the command widget.
pub fn plugin() -> omega::Plugin {
    omega::plugin!().surface(Custom)
}

#[cfg(test)]
mod tests;
