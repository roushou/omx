use omega::{
    Command, Percent,
    platform::desktop::{Backlight, Brightness},
};

/// Set the supported screen backlight, from 0% to 100%.
#[derive(Debug, omega::Command)]
pub struct SetBrightness {
    backlight: Backlight,
    brightness: Brightness,
}

impl Command for SetBrightness {
    const ID: &'static str = "display.brightness";

    type Input = Percent;
    type Output = ();

    const DESCRIPTION: &'static str = "Set the screen backlight brightness";

    async fn call(&self, level: Percent) -> omega::Result<()> {
        if !self.backlight.has_reading() {
            return Err(omega::Error::invalid("No supported backlight"));
        }

        self.brightness.set(level).await
    }
}
