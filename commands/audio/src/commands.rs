use omega::{
    Command, Percent,
    platform::audio::{Audio, Volume},
};

/// Set output volume to a percentage; unavailable audio is refused.
#[derive(Debug, omega::Command)]
pub struct SetVolume {
    audio: Audio,
    volume: Volume,
}

impl Command for SetVolume {
    const ID: &'static str = "audio.volume";

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
pub struct SetAudible {
    audio: Audio,
    volume: Volume,
}

impl Command for SetAudible {
    const ID: &'static str = "audio.audible";

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
