//! Screen backlight controls and compositor monitor readings.

use desktop_ui::{ItemRow, LevelControl, PanelHeader};
use omega::{
    Command, Percent, Surface, View,
    platform::desktop::{Backlight, Brightness, Monitors},
    surface::{Events, Task},
    ui::{Column, Component, Header, Icon, Row, Separator, Size, Text},
};
use std::convert::Infallible;

/// Bar display preferences.
#[derive(Debug, Clone, omega::Config)]
pub struct Settings {
    /// Show the backlight percentage beside the icon. Defaults to false.
    pub show_percentage: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_percentage: true,
        }
    }
}

/// A monitor icon and optional backlight percentage.
#[derive(Debug, omega::Surface)]
pub struct Indicator {
    backlight: Backlight,
    monitors: Monitors,
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
        let count = self
            .monitors
            .all()
            .iter()
            .filter(|monitor| monitor.is_connected())
            .count();
        let mut row = Row::new()
            .gap(6)
            .child(Icon::named(Display::icon(count)))
            .tooltip(if self.backlight.has_reading() {
                format!("Display · {}", self.backlight.level())
            } else {
                "Display · Backlight unavailable".into()
            });

        if self.settings.show_percentage && self.backlight.has_reading() {
            row = row.child(Text::new(self.backlight.level()));
        }

        row.into()
    }
}

/// Backlight slider and monitor resolution, refresh rate, and scale.
#[derive(Debug, omega::Surface)]
pub struct Panel {
    backlight: Backlight,
    monitors: Monitors,
}

impl Surface for Panel {
    type Model = ();
    type Message = Infallible;
    type Effects = ();

    fn update(&self, _: &mut (), message: Infallible, _: &()) -> Task<Infallible> {
        match message {}
    }

    fn render(&self, _: &(), _: &Events<Infallible>) -> View {
        let monitors: Vec<_> = self
            .monitors
            .all()
            .into_iter()
            .filter(|monitor| monitor.is_connected())
            .collect();

        let mut panel = Column::new()
            .width(352)
            .gap(14)
            .child(
                PanelHeader::labelled(
                    "Display",
                    if self.backlight.has_reading() {
                        Display::label(self.backlight.level())
                    } else {
                        "Backlight unavailable"
                    },
                )
                .leading(Icon::named(Display::icon(monitors.len())).size(Size::Display))
                .render(),
            )
            .child(Separator::new());

        if self.backlight.has_reading() {
            panel = panel.child(
                LevelControl {
                    label: "Brightness",
                    level: self.backlight.level(),
                    change: SetBrightness.into(),
                }
                .key("brightness"),
            );
        } else {
            panel = panel.child(Text::new("No supported backlight").muted());
        }

        panel = panel.child(Separator::new()).child(Header::new("DISPLAYS"));

        if !self.monitors.has_reading() {
            panel = panel.child(Text::new("Monitor information unavailable").muted());
        } else if monitors.is_empty() {
            panel = panel.child(Text::new("No connected displays").muted());
        }

        for monitor in monitors {
            panel = panel.child(
                ItemRow::new(Text::new(monitor.id()).bold())
                    .leading(Icon::named("󰍹"))
                    .subtitle(
                        Text::new(format!(
                            "{} × {} · {:.0} Hz",
                            monitor.width(),
                            monitor.height(),
                            monitor.refresh_hz()
                        ))
                        .size(Size::Caption)
                        .muted(),
                    )
                    .trailing(
                        Text::new(format!("{:.0}%", monitor.scale() * 100.0))
                            .size(Size::Caption)
                            .tooltip("Current display scale"),
                    )
                    .render()
                    .key(monitor.id()),
            );
        }

        panel.into()
    }
}

/// Set the supported screen backlight, from 0% to 100%.
#[derive(Debug, omega::Command)]
#[omega(name = "brightness")]
pub struct SetBrightness {
    backlight: Backlight,
    brightness: Brightness,
}

impl Command for SetBrightness {
    type Input = Percent;
    type Output = ();

    async fn call(&self, level: Percent) -> omega::Result<()> {
        if !self.backlight.has_reading() {
            return Err(omega::Error::invalid("No supported backlight"));
        }

        self.brightness.set(level).await
    }
}

struct Display;

impl Display {
    fn icon(count: usize) -> &'static str {
        if count > 1 { "󰍺" } else { "󰍹" }
    }

    fn label(level: Percent) -> &'static str {
        match level.whole_percent() {
            0..=9 => "Night owl",
            10..=19 => "Candlelit",
            20..=29 => "Lamp light",
            30..=44 => "Soft glow",
            45..=64 => "Even day",
            65..=79 => "Golden hour",
            80..=94 => "Solar flare",
            _ => "Sun blast",
        }
    }
}

/// Register the indicator, panel, and typed brightness control.
pub fn plugin() -> omega::Plugin {
    omega::plugin!()
        .surface(Indicator)
        .surface(Panel)
        .command::<SetBrightness>()
}

#[cfg(test)]
mod previews;

#[cfg(test)]
mod tests;
