//! Default-output volume and mute controls.

use desktop_ui::{LevelControl, PanelHeader};
use omega::{
    Command, Percent, Surface, View,
    platform::audio::{Audio, Volume},
    surface::{Events, Task},
    ui::{Column, Component, Glyph, Icon, Row, Separator, Size, Text, Toggle},
};
use std::convert::Infallible;

/// Bar display preferences. The panel always shows the current volume.
#[derive(Debug, Clone, omega::Config)]
pub struct Settings {
    /// Show the output percentage beside the icon. Defaults to false.
    pub show_percentage: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_percentage: true,
        }
    }
}

/// Speaker icon and optional output volume.
#[derive(Debug, omega::Surface)]
pub struct Indicator {
    audio: Audio,
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
        if !self.audio.has_reading() {
            return Icon::new(Glyph::VolumeOff)
                .muted()
                .tooltip("Audio unavailable")
                .into();
        }

        let level = self.audio.volume();
        let muted = self.audio.is_muted();
        let mut row = Row::new()
            .gap(6)
            .tooltip(format!("Audio · {level} · {}", Output::label(level, muted)))
            .child(Icon::new(Output::icon(level, muted)));

        if self.settings.show_percentage {
            row = row.child(Text::new(level));
        }

        row.into()
    }
}

/// A mute switch and slider for the default output.
#[derive(Debug, omega::Surface)]
pub struct Panel {
    audio: Audio,
}

impl Surface for Panel {
    type Model = ();
    type Message = Infallible;
    type Effects = ();

    fn update(&self, _: &mut (), message: Infallible, _: &()) -> Task<Infallible> {
        match message {}
    }

    fn render(&self, _: &(), _: &Events<Infallible>) -> View {
        let ready = self.audio.has_reading();
        let level = self.audio.volume();
        let muted = self.audio.is_muted();
        let mut panel = Column::new().width(352).gap(14).child(
            PanelHeader::labelled(
                "Audio",
                if ready {
                    Output::label(level, muted)
                } else {
                    "Unavailable"
                },
            )
            .leading(Icon::new(Output::icon(level, !ready || muted)).size(Size::Display))
            .trailing(if ready {
                Toggle::new(!muted)
                    .key("audible")
                    .tooltip("Mute or unmute output")
                    .on_change(SetAudible)
                    .into()
            } else {
                View::empty()
            })
            .render(),
        );

        if ready {
            panel = panel.child(Separator::new()).child(
                LevelControl {
                    label: "Output",
                    level,
                    change: SetVolume.into(),
                }
                .key("output"),
            );
        } else {
            panel = panel.child(Text::new("Audio unavailable").muted());
        }

        panel.into()
    }
}

/// Set output volume to a percentage; unavailable audio is refused.
#[derive(Debug, omega::Command)]
#[omega(name = "volume")]
pub struct SetVolume {
    audio: Audio,
    volume: Volume,
}

impl Command for SetVolume {
    type Input = Percent;
    type Output = ();

    const DESCRIPTION: &'static str = "Set the audio output volume";

    async fn call(&self, level: Percent) -> omega::Result<()> {
        if !self.audio.has_reading() {
            return Err(omega::Error::invalid("Audio unavailable"));
        }

        self.volume.set(level).await
    }
}

/// Set the output's audible state absolutely; unavailable audio is refused.
#[derive(Debug, omega::Command)]
#[omega(name = "audible")]
pub struct SetAudible {
    audio: Audio,
    volume: Volume,
}

impl Command for SetAudible {
    type Input = bool;
    type Output = ();

    const DESCRIPTION: &'static str = "Enable or mute the audio output";

    async fn call(&self, audible: bool) -> omega::Result<()> {
        if !self.audio.has_reading() {
            return Err(omega::Error::invalid("Audio unavailable"));
        }

        self.volume.set_muted(!audible).await
    }
}

struct Output;

impl Output {
    fn icon(level: Percent, muted: bool) -> Glyph {
        match level.whole_percent() {
            _ if muted => Glyph::VolumeOff,
            0 => Glyph::VolumeOff,
            1..=33 => Glyph::VolumeOff,
            34..=66 => Glyph::VolumeDown,
            _ => Glyph::VolumeUp,
        }
    }

    fn label(level: Percent, muted: bool) -> &'static str {
        match level.whole_percent() {
            _ if muted => "Muted",
            0 => "Silenced",
            1..=14 => "Whisper",
            15..=29 => "Murmur",
            30..=49 => "Easy listening",
            50..=69 => "Steady groove",
            70..=84 => "Cranked up",
            85..=99 => "Party mode",
            _ => "Concert hall",
        }
    }
}

/// Register the two surfaces and typed output controls.
pub fn plugin() -> omega::Plugin {
    omega::plugin!()
        .surface(Indicator)
        .surface(Panel)
        .command::<SetVolume>()
        .command::<SetAudible>()
}

#[cfg(test)]
mod previews;

#[cfg(test)]
mod tests;
