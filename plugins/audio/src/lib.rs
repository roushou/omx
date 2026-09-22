//! Default-output volume and mute controls.

use audio_commands::{SetAudible, SetVolume};

use omega::ui::{LevelControl, PanelHeader};
use omega::{
    Percent, Surface, View,
    platform::audio::{Audio, StreamControl, Streams},
    surface::{Events, Task},
    ui::{Column, Component, Glyph, Header, Icon, Row, Section, Separator, Size, Slider, Text, Toggle},
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

/// A mute switch and slider for the default output, plus per-application streams.
#[derive(Debug, omega::Surface)]
pub struct Panel {
    audio: Audio,
    streams: Streams,
}

/// Command dependencies used by this surface's bindings.
#[derive(Debug, omega::Effects)]
pub struct CommandEffects {
    _set_volume: omega::command::Caller<SetVolume>,
    _set_audible: omega::command::Caller<SetAudible>,
    stream_volume: StreamControl,
}

/// Local behavior for per-stream controls, whose targets are runtime data.
#[derive(Debug)]
pub enum Message {
    StreamVolume { index: u32, level: Percent },
    StreamMute { index: u32, muted: bool },
    Noop,
}

impl Surface for Panel {
    type Model = ();
    type Message = Message;
    type Effects = CommandEffects;

    fn update(&self, _: &mut (), message: Message, effects: &Self::Effects) -> Task<Message> {
        match message {
            Message::StreamVolume { index, level } => {
                Task::perform(effects.stream_volume.set_volume(index, level), |_| {
                    Message::Noop
                })
            }
            Message::StreamMute { index, muted } => {
                Task::perform(effects.stream_volume.set_muted(index, muted), |_| {
                    Message::Noop
                })
            }
            Message::Noop => Task::none(),
        }
    }

    fn render(&self, _: &(), events: &Events<Message>) -> View {
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

            let streams = self.streams.streams();
            if self.streams.has_reading() && !streams.is_empty() {
                panel = panel.child(Separator::new());
                let mut section = Section::new().heading(Header::new("APPLICATIONS"));
                for stream in streams {
                    let index = stream.index();
                    let level = stream.volume();
                    section = section.child(
                        Row::new()
                            .gap(10)
                            .child(Text::new(stream.app()).width(120).muted())
                            .child(
                                Slider::new(level)
                                    .fill_width()
                                    .on_change(events.on(move |level: Percent| {
                                        Message::StreamVolume { index, level }
                                    })),
                            )
                            .child(
                                Toggle::new(!stream.is_muted()).on_change(events.on(
                                    move |on: bool| Message::StreamMute {
                                        index,
                                        muted: !on,
                                    },
                                )),
                            )
                            .key(index.to_string()),
                    );
                }
                panel = panel.child(section);
            }
        } else {
            panel = panel.child(Text::new("Audio unavailable").muted());
        }

        panel.into()
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

/// Register the audio surfaces.
pub fn plugin() -> omega::Plugin {
    omega::plugin!().surface(Indicator).surface(Panel)
}

#[cfg(test)]
mod previews;

#[cfg(test)]
mod tests;
