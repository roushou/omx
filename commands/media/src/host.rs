use crate::{Next, Pause, Play, Previous};
use omega::host::CommandHost;

/// Declares this domain’s command exports. The system document selects deployment policy.
pub struct Host;

impl Host {
    pub fn declaration() -> CommandHost {
        CommandHost::new("media-commands", env!("CARGO_PKG_VERSION"))
            .command::<Play>()
            .command::<Pause>()
            .command::<Previous>()
            .command::<Next>()
    }
}
