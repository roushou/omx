use omega::{
    Command,
    platform::capture::{Capture, Ocr, Recording},
};

/// Copy a selected screen region to the clipboard.
#[derive(Debug, omega::Command)]
pub struct Screenshot {
    capture: Capture,
}

impl Command for Screenshot {
    const ID: &'static str = "capture.screenshot";

    type Input = ();
    type Output = ();

    const DESCRIPTION: &'static str = "Copy a selected screen region to the clipboard";

    async fn call(&self, _: ()) -> omega::Result<()> {
        self.capture.clipboard().await
    }
}

/// Capture the full screen to a file.
#[derive(Debug, omega::Command)]
pub struct ScreenshotFullscreen {
    capture: Capture,
}

impl Command for ScreenshotFullscreen {
    const ID: &'static str = "capture.fullscreen";

    type Input = String;
    type Output = ();

    const DESCRIPTION: &'static str = "Capture the full screen to a file";

    async fn call(&self, path: String) -> omega::Result<()> {
        self.capture.fullscreen(path).await
    }
}

/// Recognize text in a selected region and copy it to the clipboard.
#[derive(Debug, omega::Command)]
pub struct OcrText {
    ocr: Ocr,
}

impl Command for OcrText {
    const ID: &'static str = "capture.text";

    type Input = ();
    type Output = ();

    const DESCRIPTION: &'static str = "Copy recognized text from a selected region";

    async fn call(&self, _: ()) -> omega::Result<()> {
        self.ocr.region().await
    }
}

/// Start recording the full screen to a file.
#[derive(Debug, omega::Command)]
pub struct Record {
    recording: Recording,
}

impl Command for Record {
    const ID: &'static str = "capture.record";

    type Input = String;
    type Output = ();

    const DESCRIPTION: &'static str = "Start recording the full screen to a file";

    async fn call(&self, path: String) -> omega::Result<()> {
        self.recording.fullscreen(path).await
    }
}

/// Stop the active screen recording.
#[derive(Debug, omega::Command)]
pub struct StopRecord {
    recording: Recording,
}

impl Command for StopRecord {
    const ID: &'static str = "capture.stop";

    type Input = ();
    type Output = ();

    const DESCRIPTION: &'static str = "Stop the active screen recording";

    async fn call(&self, _: ()) -> omega::Result<()> {
        self.recording.stop().await
    }
}
