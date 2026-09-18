//! Locally defined candidates; model output can only rank these operations.
use omega::{
    Args, Input, Percent,
    command::{Available, CommandRef},
    config::{IntoValue, Value},
    platform::{
        applications::{Application, ApplicationId},
        audio::{Media, PlayerId},
        bluetooth::{Bluetooth, DeviceId},
    },
};
use std::{collections::BTreeSet, fmt};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CandidateId {
    Application(ApplicationId),
    Action(String),
}
impl fmt::Display for CandidateId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Application(id) => id.fmt(f),
            Self::Action(id) => write!(f, "command:{id}"),
        }
    }
}
impl From<ApplicationId> for CandidateId {
    fn from(id: ApplicationId) -> Self {
        Self::Application(id)
    }
}
impl Input for CandidateId {
    fn decode(args: Args) -> omega::Result<Self> {
        let text = String::decode(args)?;
        match text.strip_prefix("command:") {
            Some(id) if !id.is_empty() && id.len() <= 1024 => Ok(Self::Action(id.into())),
            Some(_) => Err(omega::Error::invalid("invalid candidate ID")),
            None => Ok(Self::Application(text.parse().map_err(
                |e: omega::platform::applications::ApplicationIdError| {
                    omega::Error::invalid(e.to_string())
                },
            )?)),
        }
    }
    fn encode(self) -> Vec<Value> {
        vec![self.to_string().into_value()]
    }
}
#[derive(Debug, Clone)]
pub(crate) enum Action {
    Audible(bool),
    Volume(u8),
    Brightness(u8),
    Connect(DeviceId),
    Disconnect(DeviceId),
    Play(PlayerId),
    Pause(PlayerId),
    Next(PlayerId),
}
impl Action {
    fn id(&self) -> String {
        match self {
            Self::Audible(v) => format!("audio/audible/{v}"),
            Self::Volume(v) => format!("audio/volume/{v}"),
            Self::Brightness(v) => format!("display/brightness/{v}"),
            Self::Connect(id) => format!("bluetooth/connect/{id}"),
            Self::Disconnect(id) => format!("bluetooth/disconnect/{id}"),
            Self::Play(id) => format!("media/play/{id}"),
            Self::Pause(id) => format!("media/pause/{id}"),
            Self::Next(id) => format!("media/next/{id}"),
        }
    }
    pub(crate) fn execute(&self, effects: &crate::Effects) -> omega::effect::Effect {
        match self {
            Self::Audible(value) => effects.audible.call(*value),
            Self::Volume(value) => effects.volume.call(Percent::whole(*value)),
            Self::Brightness(value) => effects.brightness.call(Percent::whole(*value)),
            Self::Connect(id) => effects.connect.call(id.clone()),
            Self::Disconnect(id) => effects.disconnect.call(id.clone()),
            Self::Play(id) => effects.play.call(id.clone()),
            Self::Pause(id) => effects.pause.call(id.clone()),
            Self::Next(id) => effects.next.call(id.clone()),
        }
    }
}
#[derive(Debug, Clone)]
enum Target {
    Application(Application),
    Action(Action),
}
#[derive(Debug, Clone)]
pub(crate) struct Candidate {
    id: CandidateId,
    title: String,
    description: String,
    target: Target,
}
impl Candidate {
    pub(crate) fn id(&self) -> &CandidateId {
        &self.id
    }
    pub(crate) fn name(&self) -> &str {
        &self.title
    }
    pub(crate) fn description(&self) -> &str {
        &self.description
    }
    pub(crate) fn application(&self) -> Option<&ApplicationId> {
        match &self.target {
            Target::Application(app) => Some(app.id()),
            Target::Action(_) => None,
        }
    }
    pub(crate) fn icon(&self) -> &str {
        match &self.target {
            Target::Application(app) => app.icon(),
            Target::Action(_) => "",
        }
    }
    pub(crate) fn is_terminal(&self) -> bool {
        matches!(&self.target, Target::Application(app) if app.is_terminal())
    }
    pub(crate) fn generic_name(&self) -> &str {
        match &self.target {
            Target::Application(app) => app.generic_name(),
            Target::Action(_) => "Desktop action",
        }
    }
    pub(crate) fn keywords(&self) -> &[String] {
        match &self.target {
            Target::Application(app) => app.keywords(),
            Target::Action(_) => &[],
        }
    }
    pub(crate) fn is_favorite(&self, favorites: &BTreeSet<ApplicationId>) -> bool {
        self.application().is_some_and(|id| favorites.contains(id))
    }
    pub(crate) fn execute(&self, effects: &crate::Effects) -> omega::effect::Effect {
        match &self.target {
            Target::Application(app) => effects.launcher.launch(app.id()),
            Target::Action(action) => action.execute(effects),
        }
    }
    pub(crate) fn semantic(&self) -> typesafe::Candidate {
        typesafe::Candidate {
            id: self.id.to_string(),
            label: self.title.clone(),
            description: format!(
                "{}; {}; {}",
                self.description,
                self.generic_name(),
                self.keywords().join(", ")
            ),
        }
    }
    fn action(action: Action, title: String, description: &str) -> Self {
        Self {
            id: CandidateId::Action(action.id()),
            title,
            description: description.into(),
            target: Target::Action(action),
        }
    }
}
impl From<Application> for Candidate {
    fn from(app: Application) -> Self {
        Self {
            id: app.id().clone().into(),
            title: app.name().into(),
            description: app.description().into(),
            target: Target::Application(app),
        }
    }
}
pub(crate) struct Sources;
impl Sources {
    fn available<C: omega::Command>(catalogue: &[Available], command: CommandRef<C>) -> bool {
        catalogue.iter().any(|entry| {
            entry.available
                && entry.address.plugin.as_str() == command.plugin()
                && entry.address.command.as_str() == command.name()
        })
    }
    pub(crate) fn collect(
        apps: Vec<Application>,
        catalogue: &[Available],
        media: &Media,
        bluetooth: &Bluetooth,
    ) -> Vec<Candidate> {
        let mut entries: Vec<_> = apps.into_iter().map(Candidate::from).collect();
        if Self::available(catalogue, audio::SetAudible) {
            entries.push(Candidate::action(
                Action::Audible(false),
                "Mute sound".into(),
                "Silence the audio output",
            ));
            entries.push(Candidate::action(
                Action::Audible(true),
                "Unmute sound".into(),
                "Enable the audio output",
            ));
        }
        for level in [25, 50, 75] {
            if Self::available(catalogue, audio::SetVolume) {
                entries.push(Candidate::action(
                    Action::Volume(level),
                    format!("Set volume to {level}%"),
                    "Adjust audio output volume",
                ));
            }
            if Self::available(catalogue, display::SetBrightness) {
                entries.push(Candidate::action(
                    Action::Brightness(level),
                    format!("Set brightness to {level}%"),
                    "Adjust screen backlight brightness",
                ));
            }
        }
        for player in media.players() {
            if player.can_play() && Self::available(catalogue, media::Play) {
                entries.push(Candidate::action(
                    Action::Play(player.id().clone()),
                    format!("Play {}", player.identity()),
                    "Start music or media playback",
                ));
            }
            if player.can_pause() && Self::available(catalogue, media::Pause) {
                entries.push(Candidate::action(
                    Action::Pause(player.id().clone()),
                    format!("Pause {}", player.identity()),
                    "Pause music or media playback",
                ));
            }
            if player.can_go_next() && Self::available(catalogue, media::Next) {
                entries.push(Candidate::action(
                    Action::Next(player.id().clone()),
                    format!("Next track in {}", player.identity()),
                    "Skip to the next media track",
                ));
            }
        }
        for device in bluetooth.known_devices() {
            if device.is_connected() && Self::available(catalogue, bluetooth::Disconnect) {
                entries.push(Candidate::action(
                    Action::Disconnect(device.id().clone()),
                    format!("Disconnect {}", device.name()),
                    "Disconnect a Bluetooth device",
                ));
            } else if device.can_connect() && Self::available(catalogue, bluetooth::Connect) {
                entries.push(Candidate::action(
                    Action::Connect(device.id().clone()),
                    format!("Connect {}", device.name()),
                    "Connect a paired Bluetooth device",
                ));
            }
        }
        entries
    }
}
