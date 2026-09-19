use crate::{SetAudible, SetVolume};
use omega::host::CommandHost;

/// Declares this domain’s command exports. The system document selects deployment policy.
pub struct Host;

impl Host {
    pub fn declaration() -> CommandHost {
        CommandHost::new("audio-commands", env!("CARGO_PKG_VERSION"))
            .command::<SetVolume>()
            .command::<SetAudible>()
    }
}
