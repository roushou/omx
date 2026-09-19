use omega::{
    Command,
    platform::audio::{Media, MediaControl, Player, PlayerId},
};

/// Play on the specified player; unavailable or unsupported targets are refused.
#[derive(Debug, omega::Command)]
pub struct Play {
    media: Media,
    control: MediaControl,
}

impl Command for Play {
    const ID: &'static str = "media.play";

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
pub struct Pause {
    media: Media,
    control: MediaControl,
}

impl Command for Pause {
    const ID: &'static str = "media.pause";

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
pub struct Previous {
    media: Media,
    control: MediaControl,
}

impl Command for Previous {
    const ID: &'static str = "media.previous";

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
pub struct Next {
    media: Media,
    control: MediaControl,
}

impl Command for Next {
    const ID: &'static str = "media.next";

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

struct Players;

impl Players {
    fn find(media: &Media, id: &PlayerId) -> omega::Result<Player> {
        media
            .players()
            .into_iter()
            .find(|p| p.id() == id)
            .ok_or_else(|| omega::Error::invalid("This media player is no longer available"))
    }
}
