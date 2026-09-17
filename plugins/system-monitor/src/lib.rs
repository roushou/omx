//! Read-only CPU, memory, filesystem, and thermal status.

use desktop_ui::{Detail, LabelledControl, PanelHeader, Section};
use omega::{
    Percent, Surface, View,
    platform::system::{Disk, Memory, System, Thermals},
    surface::{Events, Task},
    ui::{Column, Component, Glyph, Header, Icon, Progress, Row, Separator, Size, Text},
};
use std::convert::Infallible;

/// Display preferences; these do not change broker sampling rates.
#[derive(Debug, Clone, omega::Config)]
pub struct Settings {
    /// Show used RAM alongside CPU in the bar. Defaults to true.
    pub show_memory: bool,
    /// Include per-core CPU readings in the panel. Defaults to false.
    pub show_cores: bool,
    /// Exact mount paths to show. Defaults to / and /home.
    pub mounts: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_memory: true,
            show_cores: false,
            mounts: vec!["/".into(), "/home".into()],
        }
    }
}

/// CPU and optionally RAM usage. Missing readings never appear as zero usage.
#[derive(Debug, omega::Surface)]
pub struct Indicator {
    system: System,
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
        if !self.system.has_reading() {
            return Icon::named("󰍛")
                .tooltip("System readings unavailable")
                .muted()
                .into();
        }

        let cpu = self.system.cpu();
        let mut row = Row::new()
            .gap(6)
            .tooltip(format!(
                "CPU {cpu} · RAM {} / {}",
                self.system.memory().used(),
                self.system.memory().total()
            ))
            .child(Icon::new(Glyph::Cpu))
            .child(Text::new(cpu));

        if self.settings.show_memory
            && let Some(memory) = self.system.memory().share()
        {
            row = row.child(Icon::named("󰍛")).child(Text::new(memory));
        }

        row.into()
    }
}

/// Resource usage, configured filesystems, sensor temperatures, and fan speeds.
#[derive(Debug, omega::Surface)]
pub struct Panel {
    system: System,
    disk: Disk,
    thermals: Thermals,
    #[omega(config)]
    settings: Settings,
}

impl Surface for Panel {
    type Model = ();
    type Message = Infallible;
    type Effects = ();

    fn update(&self, _: &mut (), message: Infallible, _: &()) -> Task<Infallible> {
        match message {}
    }

    fn render(&self, _: &(), _: &Events<Infallible>) -> View {
        let status = if self.system.has_reading() {
            format!("Uptime · {}", self.system.uptime())
        } else {
            "Readings unavailable".into()
        };

        let panel = Column::new().width(352).gap(14).child(
            PanelHeader::labelled("System", &status)
                .leading(Icon::named("󰍛").size(Size::Display))
                .render(),
        );

        panel
            .children(self.resources())
            .child(Separator::new())
            .child(self.storage())
            .child(Separator::new())
            .child(self.thermals())
            .into()
    }
}

impl Panel {
    fn resources(&self) -> Vec<View> {
        let mut views = Vec::new();
        if self.system.has_reading() {
            views.push(Separator::new().into());
            views.push(Meters::usage("CPU", self.system.cpu()));

            if self.settings.show_cores {
                for (index, level) in self.system.cores().into_iter().enumerate() {
                    views.push(Detail::row(format!("Core {index}"), level));
                }
            }

            let load = self.system.load();
            views.push(Detail::row(
                "Load · 1 / 5 / 15 min",
                format!("{:.2} / {:.2} / {:.2}", load.one, load.five, load.fifteen),
            ));
            views.push(Meters::memory("Memory", self.system.memory()));
            views.push(if self.system.swap().share().is_some() {
                Meters::memory("Swap", self.system.swap())
            } else {
                Detail::row("Swap", "Not configured")
            });
        } else {
            views.push(Text::new("CPU and memory unavailable").muted().into());
        }

        views
    }

    fn storage(&self) -> View {
        let mut storage = Section::new().heading(Header::new("STORAGE"));

        if !self.disk.has_reading() {
            storage = storage.child(Text::new("Storage readings unavailable").muted());
        } else if self.settings.mounts.is_empty() {
            storage = storage.child(Text::new("No mounts selected").muted());
        } else {
            let mut seen = std::collections::BTreeSet::new();

            for path in &self.settings.mounts {
                if !seen.insert(path) {
                    continue;
                }

                if let Some(mount) = self.disk.at(path) {
                    let mut section = Column::new().gap(6).key(path).child(Detail::row(
                        path,
                        format!("{} / {}", mount.used(), mount.total()),
                    ));

                    if let Some(share) = mount.share() {
                        section = section.child(Progress::new(share));
                    }

                    section = section.child(
                        Text::new(format!(
                            "{} · {} available",
                            mount.filesystem(),
                            mount.available()
                        ))
                        .size(Size::Caption)
                        .muted(),
                    );
                    storage = storage.child(section);
                } else {
                    storage = storage.child(Detail::row(path, "Not reported"));
                }
            }
        }

        storage.render()
    }

    fn thermals(&self) -> View {
        let mut thermals = Section::new().heading(Header::new("THERMALS"));
        let sensors = self.thermals.sensors();
        let fans = self.thermals.fans();

        if !self.thermals.has_reading() {
            thermals = thermals.child(Text::new("Thermal readings unavailable").muted());
        } else if sensors.is_empty() && fans.is_empty() {
            thermals = thermals.child(Text::new("No readable sensors").muted());
        }

        for sensor in sensors {
            thermals = thermals.child(Detail::row(
                format!("{} · {}", sensor.chip(), sensor.label()),
                sensor.temperature(),
            ));
        }

        for fan in fans {
            thermals = thermals.child(Detail::row(
                format!("{} · {}", fan.chip(), fan.label()),
                format!("{} RPM", fan.rpm()),
            ));
        }

        thermals.render()
    }
}

struct Meters;

impl Meters {
    fn usage(label: &str, level: Percent) -> View {
        LabelledControl::new(
            Text::new(label).muted(),
            Text::new(level),
            Progress::new(level),
        )
        .gap(6)
        .render()
    }

    fn memory(label: &str, memory: Memory) -> View {
        let progress: View = match memory.share() {
            Some(share) => Progress::new(share).into(),
            None => View::empty(),
        };
        LabelledControl::new(
            Text::new(label).muted(),
            Text::new(format!("{} / {}", memory.used(), memory.total())),
            progress,
        )
        .gap(6)
        .render()
    }
}

/// Register read-only bar and panel surfaces.
pub fn plugin() -> omega::Plugin {
    omega::plugin!().surface(Indicator).surface(Panel)
}

#[cfg(test)]
mod previews;

#[cfg(test)]
mod tests;
