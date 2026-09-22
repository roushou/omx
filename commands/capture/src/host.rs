use crate::{OcrText, Record, Screenshot, ScreenshotFullscreen, StopRecord};
use omega::host::CommandHost;

/// Declares this domain's command exports. The system document selects deployment policy.
pub struct Host;

impl Host {
    pub fn declaration() -> CommandHost {
        CommandHost::new("capture-commands", env!("CARGO_PKG_VERSION"))
            .command::<Screenshot>()
            .command::<ScreenshotFullscreen>()
            .command::<OcrText>()
            .command::<Record>()
            .command::<StopRecord>()
    }
}
