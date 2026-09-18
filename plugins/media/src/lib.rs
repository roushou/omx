//! Player-specific media controls with instance-local player selection.

use desktop_ui::PanelHeader;
use omega::{
    Command, Surface, View,
    platform::audio::{Media, MediaControl, Playback, Player, PlayerId},
    surface::{Events, Task},
    ui::{Button, Column, Component, Glyph, Icon, List, Row, Separator, Size, Text},
};
use std::convert::Infallible;

/// Media bar preferences.
#[derive(Debug, Clone, omega::Config)]
pub struct Settings {
    /// Include the current title in the bar. Defaults to true.
    pub show_title: bool,
    /// Maximum title characters, clamped to 8–80. Defaults to 28.
    pub title_chars: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_title: true,
            title_chars: 28,
        }
    }
}

/// Current player summary; draws nothing when no player is available.
#[derive(Debug, omega::Surface)]
pub struct Indicator {
    media: Media,
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
        let Some(player) = Players::default(&self.media) else {
            return View::empty();
        };

        let title = Players::title(&player);
        let mut row = Row::new()
            .gap(6)
            .tooltip(format!(
                "{} · {} · {}",
                player.identity(),
                title,
                Players::status(player.playback())
            ))
            .child(Icon::new(if player.is_playing() {
                Glyph::Music
            } else {
                Glyph::Pause
            }));

        if self.settings.show_title {
            row = row.child(Text::new(Players::shorten(
                title,
                self.settings.title_chars,
            )));
        }

        row.into()
    }
}

/// The selected player's ID belongs to this panel instance.
#[derive(Debug, Default)]
pub struct Model {
    selected: Option<PlayerId>,
}

/// Select a player by the list's validated item ID.
#[derive(Debug)]
pub enum Message {
    Select(String),
}

/// Player picker, track metadata, and capability-aware transport controls.
#[derive(Debug, omega::Surface)]
pub struct Panel {
    media: Media,
}

impl Surface for Panel {
    type Model = Model;
    type Message = Message;
    type Effects = ();

    fn update(&self, model: &mut Model, message: Message, _: &()) -> Task<Message> {
        match message {
            Message::Select(id) => {
                if let Ok(id) = PlayerId::try_from(id)
                    && self.media.players().iter().any(|p| p.id() == &id)
                {
                    model.selected = Some(id)
                }
            }
        }

        Task::none()
    }

    fn render(&self, model: &Model, events: &Events<Message>) -> View {
        let players = self.media.players();
        let selected = match &model.selected {
            Some(id) => players.iter().find(|p| p.id() == id).cloned(),
            None => Players::default(&self.media),
        };

        let mut panel = Column::new().width(352).gap(14).child(
            PanelHeader::labelled(
                "Media",
                if self.media.has_reading() {
                    "Players"
                } else {
                    "Unavailable"
                },
            )
            .leading(Icon::new(Glyph::Music).size(Size::Display))
            .render(),
        );

        if players.is_empty() {
            return panel
                .child(
                    Text::new(if self.media.has_reading() {
                        "No media players"
                    } else {
                        "Media unavailable"
                    })
                    .muted(),
                )
                .into();
        }

        panel = panel.child(Self::player_list(&players, selected.as_ref(), events));

        let Some(player) = selected else {
            return panel
                .child(Text::new("The selected player closed. Choose another player.").muted())
                .into();
        };

        panel.children(Self::player_details(&player)).into()
    }
}

impl Panel {
    fn player_list(
        players: &[Player],
        selected: Option<&Player>,
        events: &Events<Message>,
    ) -> View {
        List::new()
            .key("players")
            .height((players.len().min(4) * 38) as u32)
            .gap(2)
            .selected(
                selected
                    .as_ref()
                    .map(|p| p.id().to_string())
                    .unwrap_or_default(),
            )
            .on_select(events.on(Message::Select))
            .on_activate(events.on(Message::Select))
            .children(players.iter().map(|player| {
                Row::new()
                    .key(player.id().as_str())
                    .gap(8)
                    .child(Icon::new(if player.is_playing() {
                        Glyph::Play
                    } else {
                        Glyph::Music
                    }))
                    .child(Text::new(player.identity()).fill_width())
                    .child(
                        Text::new(Players::status(player.playback()))
                            .size(Size::Caption)
                            .muted(),
                    )
            }))
            .into()
    }

    fn player_details(player: &Player) -> Vec<View> {
        let id = player.id().clone();
        let mut views = vec![
            Separator::new().into(),
            Text::new(Players::title(player))
                .size(Size::Title)
                .bold()
                .into(),
        ];

        if !player.artist().is_empty() {
            views.push(Text::new(player.artist()).into());
        }

        if !player.album().is_empty() {
            views.push(Text::new(player.album()).muted().into());
        }

        if let Some(length) = player.length() {
            views.push(
                Text::new(format!("Duration · {length}"))
                    .size(Size::Caption)
                    .muted()
                    .into(),
            );
        }

        let playback = if player.is_playing() {
            Button::new("Pause")
                .key("playback")
                .disabled_if(!player.can_pause())
                .on_press(Pause.with(id.clone()))
        } else {
            Button::new("Play")
                .key("playback")
                .disabled_if(!player.can_play())
                .on_press(Play.with(id.clone()))
        };
        views.push(
            Row::new()
                .gap(8)
                .child(
                    Button::new("Previous")
                        .key("previous")
                        .secondary()
                        .disabled_if(!player.can_go_previous())
                        .on_press(Previous.with(id.clone()))
                        .fill_width(),
                )
                .child(playback.fill_width())
                .child(
                    Button::new("Next")
                        .key("next")
                        .secondary()
                        .disabled_if(!player.can_go_next())
                        .on_press(Next.with(id))
                        .fill_width(),
                )
                .into(),
        );
        views
    }
}

struct Players;

impl Players {
    fn default(media: &Media) -> Option<Player> {
        media
            .active()
            .or_else(|| media.players().into_iter().next())
    }

    fn find(media: &Media, id: &PlayerId) -> omega::Result<Player> {
        media
            .players()
            .into_iter()
            .find(|p| p.id() == id)
            .ok_or_else(|| omega::Error::invalid("This media player is no longer available"))
    }

    fn title(player: &Player) -> &str {
        if player.title().is_empty() {
            player.identity()
        } else {
            player.title()
        }
    }

    fn status(playback: Playback) -> &'static str {
        match playback {
            Playback::Playing => "Playing",
            Playback::Paused => "Paused",
            Playback::Stopped => "Stopped",
            Playback::Unspecified => "Unknown",
        }
    }

    fn shorten(title: &str, limit: u8) -> String {
        let limit = usize::from(limit.clamp(8, 80));

        if title.chars().count() <= limit {
            title.into()
        } else {
            format!("{}…", title.chars().take(limit - 1).collect::<String>())
        }
    }
}

/// Play on the specified player; unavailable or unsupported targets are refused.
#[derive(Debug, omega::Command)]
#[omega(name = "play")]
pub struct Play {
    media: Media,
    control: MediaControl,
}

impl Command for Play {
    type Input = PlayerId;
    type Output = ();

    const DESCRIPTION: &'static str = "Start playback in a media player";

    async fn call(&self, id: PlayerId) -> omega::Result<()> {
        let player = Players::find(&self.media, &id)?;

        if !player.can_play() {
            return Err(omega::Error::invalid("This player does not support play"));
        }

        self.control.player(&id).play().await
    }
}

/// Pause on the specified player; unavailable or unsupported targets are refused.
#[derive(Debug, omega::Command)]
#[omega(name = "pause")]
pub struct Pause {
    media: Media,
    control: MediaControl,
}

impl Command for Pause {
    type Input = PlayerId;
    type Output = ();

    const DESCRIPTION: &'static str = "Pause a media player";

    async fn call(&self, id: PlayerId) -> omega::Result<()> {
        let player = Players::find(&self.media, &id)?;

        if !player.can_pause() {
            return Err(omega::Error::invalid("This player does not support pause"));
        }

        self.control.player(&id).pause().await
    }
}

/// Previous on the specified player; unavailable or unsupported targets are refused.
#[derive(Debug, omega::Command)]
#[omega(name = "previous")]
pub struct Previous {
    media: Media,
    control: MediaControl,
}

impl Command for Previous {
    type Input = PlayerId;
    type Output = ();

    const DESCRIPTION: &'static str = "Return to the previous track";

    async fn call(&self, id: PlayerId) -> omega::Result<()> {
        let player = Players::find(&self.media, &id)?;

        if !player.can_go_previous() {
            return Err(omega::Error::invalid(
                "This player does not support previous",
            ));
        }

        self.control.player(&id).previous().await
    }
}

/// Next on the specified player; unavailable or unsupported targets are refused.
#[derive(Debug, omega::Command)]
#[omega(name = "next")]
pub struct Next {
    media: Media,
    control: MediaControl,
}

impl Command for Next {
    type Input = PlayerId;
    type Output = ();

    const DESCRIPTION: &'static str = "Skip to the next track";

    async fn call(&self, id: PlayerId) -> omega::Result<()> {
        let player = Players::find(&self.media, &id)?;

        if !player.can_go_next() {
            return Err(omega::Error::invalid("This player does not support next"));
        }

        self.control.player(&id).next().await
    }
}

/// Register the bar, panel, and player-specific commands.
pub fn plugin() -> omega::Plugin {
    omega::plugin!()
        .surface(Indicator)
        .surface(Panel)
        .command::<Play>()
        .command::<Pause>()
        .command::<Previous>()
        .command::<Next>()
}

#[cfg(test)]
mod previews;

#[cfg(test)]
mod tests;
