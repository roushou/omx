//! Numbered workspace buttons with compositor-owned focus and occupancy.

mod slots;

use omega::{
    Command, Surface, View,
    platform::desktop::{WorkspaceControl, WorkspaceIndex, Workspaces},
    surface::{Events, Task},
    ui::{Button, Column, Glyph, Icon, Row},
};
use slots::Slots;
use std::convert::Infallible;

/// Numbered workspace display preferences.
#[derive(Debug, Clone, omega::Config)]
pub struct Settings {
    /// Always show 1 through this number, even when unopened. Defaults to 5.
    /// Clamped to the effective maximum; zero disables persistent slots.
    pub persistent: u8,
    /// Highest displayed workspace, clamped to 1–30. Defaults to 10.
    pub maximum: u8,
    /// Stack buttons vertically. Defaults to false for a horizontal bar.
    pub vertical: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            persistent: 5,
            maximum: 10,
            vertical: false,
        }
    }
}

/// A global workspace list, identical on each output's bar.
#[derive(Debug, omega::Surface)]
pub struct Indicator {
    workspaces: Workspaces,
    #[omega(config)]
    settings: Settings,
}

impl Surface for Indicator {
    type Model = ();
    type Message = Infallible;
    type Effects = ();

    fn update(&self, _: &mut (), message: Infallible, _: &()) -> Task<Infallible> {
        match message {}
    }

    fn render(&self, _: &(), _: &Events<Infallible>) -> View {
        if !self.workspaces.has_reading() {
            return Icon::new(Glyph::Warning)
                .muted()
                .tooltip("Workspaces unavailable")
                .into();
        }

        let row = if self.settings.vertical {
            Column::new()
        } else {
            Row::new()
        };

        row.gap(1)
            .children(
                Slots::of(&self.workspaces.all(), &self.settings)
                    .into_iter()
                    .map(|slot| {
                        let label = if slot.focused {
                            "●".into()
                        } else if slot.index.get() == 10 {
                            "0".into()
                        } else {
                            slot.index.to_string()
                        };

                        let button = Button::new(label)
                            .flat()
                            .width(20)
                            .height(24)
                            .key(format!("workspace-{}", slot.index))
                            .tooltip(slot.tooltip())
                            .on_press(Select.with(slot.index));

                        if slot.focused {
                            button.primary()
                        } else if slot.windows == 0 {
                            button.muted()
                        } else {
                            button
                        }
                    }),
            )
            .into()
    }
}

/// Focus a numbered workspace, creating it if the compositor supports it.
/// Does not require a current reading or optimistically change the highlight.
#[derive(Debug, omega::Command)]
#[omega(name = "select")]
pub struct Select {
    control: WorkspaceControl,
}

impl Command for Select {
    type Input = WorkspaceIndex;
    type Output = ();

    async fn call(&self, index: WorkspaceIndex) -> omega::Result<()> {
        self.control.switch_to(index).await
    }
}

/// Register the indicator and typed workspace-selection command.
pub fn plugin() -> omega::Plugin {
    omega::plugin!().surface(Indicator).command::<Select>()
}

#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod previews;
#[cfg(test)]
mod tests;
