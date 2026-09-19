use crate::ChangeProfile;
use omega::host::CommandHost;

/// Declares this domain’s command exports. The system document selects deployment policy.
pub struct Host;

impl Host {
    pub fn declaration() -> CommandHost {
        CommandHost::new("power-commands", env!("CARGO_PKG_VERSION")).command::<ChangeProfile>()
    }
}
