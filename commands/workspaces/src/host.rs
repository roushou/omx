use crate::Select;
use omega::host::CommandHost;

/// Declares this domain’s command exports. The system document selects deployment policy.
pub struct Host;

impl Host {
    pub fn declaration() -> CommandHost {
        CommandHost::new("workspaces-commands", env!("CARGO_PKG_VERSION")).command::<Select>()
    }
}
